/*
 * Copyright 2021-2023 Golden_Water
 * SPDX-License-Identifier: AGPL-3.0-only
 */

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use itertools::Itertools;
use log::{debug, error, info, warn};
use syn::Item;

use crate::item::item_info::ItemInfo;
use crate::item::item_parser::ItemParser;
use crate::ts_compiler::TsCompiler;
use crate::use_path::{PathFs, PathRs, UsePath};
use crate::utils::ts_ast_utils::create_import;
use crate::{TEError, TEResult};

pub struct TypeExporter {
  root: PathBuf,
  output: PathBuf,

  compiler: TsCompiler,

  items: HashMap<UsePath<PathFs>, Vec<TEResult<ItemInfo>>>,
}

impl TypeExporter {
  pub fn new(root: PathBuf, output: PathBuf) -> TEResult<Self> {
    let cargo_toml = root.join("Cargo.toml");
    let src_dir = root.join("src");

    if !(cargo_toml.exists() && src_dir.exists()) {
      return Err(TEError::InvalidCargoProjectRoot);
    }

    Ok(Self {
      root: src_dir.canonicalize()?,
      compiler: TsCompiler::default(),
      output: output.canonicalize()?,
      items: HashMap::new(),
    })
  }

  pub fn run(root: PathBuf, output: PathBuf) -> TEResult<()> {
    let mut type_exporter = Self::new(root, output)?;

    type_exporter.scan_and_parse_files()?;
    type_exporter.transform_and_write();

    Ok(())
  }

  pub fn scan_and_parse_files(&mut self) -> TEResult<()> {
    info!("scan and parse");
    self.items = walkdir::WalkDir::new(&self.root)
      .into_iter()
      .collect::<walkdir::Result<Vec<_>>>()?
      .into_iter()
      .filter(|it| it.file_type().is_file())
      .filter(|it| !it.file_name().to_string_lossy().starts_with('.'))
      .filter(|it| it.file_name().to_string_lossy().ends_with(".rs"))
      .map(|it| it.path().strip_prefix(&self.root).unwrap().to_path_buf())
      .map(|it| self.parse_file(it))
      .filter(|it| match it {
        Ok(v) => !v.1.is_empty(),
        Err(_) => true,
      })
      .collect::<TEResult<HashMap<_, _>>>()?;

    Ok(())
  }

  fn parse_file(&self, fs_path: PathBuf) -> TEResult<(UsePath<PathFs>, Vec<TEResult<ItemInfo>>)> {
    info!("loading {fs_path:?}");
    let data = fs::read_to_string(self.root.join(&fs_path))?;

    debug!("parsing");
    let path = UsePath::<PathFs>::from(fs_path);
    let data = syn::parse_file(&data)?;

    let uses = data
      .items
      .iter()
      .filter_map(|it| match it {
        Item::Use(it) => Some(it),
        _ => None,
      })
      .flat_map(UsePath::<PathRs>::parse_item_use)
      .filter(|it| it.is_absolute() || it.is_start_with_parent())
      .collect::<Vec<_>>();
    let local_types = data
      .items
      .iter()
      .filter_map(|item| match item {
        Item::Enum(it) => Some(it.ident.to_string()),
        Item::Struct(it) => Some(it.ident.to_string()),
        _ => None,
      })
      .collect::<Vec<_>>();

    let parser = ItemParser::new(&uses, &local_types);

    let result = data
      .items
      .into_iter()
      .filter_map(|item| parser.parse_item(&path.to_rs(), &item))
      .collect::<Vec<_>>();

    Ok((path, result))
  }

  pub fn transform_and_write(&self) {
    info!("transform and write");
    let mut deps = self.transform_and_write_files(self.collect_entries());

    while !deps.is_empty() {
      deps = self.transform_and_write_files(deps);
    }
  }

  fn transform_and_write_files(
    &self,
    files: HashMap<&UsePath<PathFs>, Vec<&ItemInfo>>,
  ) -> HashMap<&UsePath<PathFs>, Vec<&ItemInfo>> {
    files
      .into_iter()
      .flat_map(|(path, items)| self.transform_and_write_file(path, items))
      .collect()
  }

  fn transform_and_write_file(
    &self,
    path: &UsePath<PathFs>,
    items: Vec<&ItemInfo>,
  ) -> HashMap<&UsePath<PathFs>, Vec<&ItemInfo>> {
    let mut dependencies = HashSet::new();

    let mut content_items = vec![];

    for item in items {
      info!(
        "transforming {} in {:?}",
        item.get_name(),
        path.to_path_buf_with_ext()
      );

      match item {
        ItemInfo::Struct { item, processed } => {
          if *processed {
            continue;
          }
          let ts_ast = item.to_ts_ast();
          content_items.push(ts_ast.ast);
          dependencies.extend(ts_ast.dependencies);
        }
        ItemInfo::Enum { item, processed } => {
          if *processed {
            continue;
          }
          let ts_ast = item.to_ts_ast();
          content_items.extend(ts_ast.ast);
          dependencies.extend(ts_ast.dependencies);
        }
      }
    }

    let mut content = dependencies
      .iter()
      .filter(|it| !it.local_use)
      .map(|it| create_import(&it.relative_from(path)))
      .collect::<Vec<_>>();

    content.extend(content_items);

    let mut output_file = self.output.join(PathBuf::from(path.clone()));
    fs::create_dir_all(output_file.parent().unwrap()).expect("failed to create dir all");
    output_file.set_extension("d.ts");
    fs::OpenOptions::new()
      .write(true)
      .append(true)
      .create(true)
      .open(output_file)
      .expect("failed to open file for writing")
      .write_all(self.compiler.compile(content).as_ref())
      .expect("failed to write");

    dependencies
      .iter()
      .group_by(|it| &it.path)
      .into_iter()
      .map(|(key, group)| {
        let path = UsePath::<PathFs>::new(key.clone())
          .to_relative()
          .to_path_buf_with_ext();
        debug!("resolving items in {path:?}");

        let canonical_path = self.root.join(&path).canonicalize()?;
        let path = canonical_path
          .strip_prefix(&self.root)
          .map(UsePath::<PathFs>::from)
          .unwrap_or_else(|_| UsePath::<PathFs>::from(path));

        // get items by file path
        if let Some((key, items)) = self.items.get_key_value(&path) {
          let mut result = vec![];

          // get items that needed
          for use_item in group {
            let name = use_item
              .actual_name
              .clone()
              .unwrap_or_else(|| use_item.name.clone());

            let value = items.iter().find(|it| match it {
              Ok(it) => it.get_name().eq(&name),
              Err(it) => match it {
                TEError::ParseItemFailed(err_name, _) => name.eq(err_name),
                _ => false,
              },
            });

            if let Some(value) = value {
              match value {
                Ok(value) => {
                  result.push(value);
                }
                Err(err) => {
                  error!("unable to transform {name} because: {err}");
                }
              }
            } else {
              warn!("failed to find {name} in {canonical_path:?}");
            }
          }

          Ok(Some((key, result)))
        } else {
          Ok(None)
        }
      })
      .filter_map(|it: TEResult<Option<_>>| match it {
        Ok(value) => value,
        Err(err) => {
          error!("{err:?}");
          None
        }
      })
      .filter(|(_, value)| !value.is_empty())
      .collect::<HashMap<_, _>>()
  }

  fn collect_entries(&self) -> HashMap<&UsePath<PathFs>, Vec<&ItemInfo>> {
    self
      .items
      .iter()
      .map(|(path, items)| {
        let entries = items
          .iter()
          .filter_map(|it| it.as_ref().ok())
          .filter(|it| match it {
            ItemInfo::Struct { item, .. } => item.attr.is_entry(),
            ItemInfo::Enum { item, .. } => item.attr.is_entry(),
          })
          .collect::<Vec<_>>();
        (path, entries)
      })
      .filter(|it| !it.1.is_empty())
      .map(|it| {
        let (path, items) = &it;

        for item in items {
          info!(
            "detected entry {}, in {:?}",
            item.get_name(),
            path.to_path_buf_with_ext()
          )
        }

        it
      })
      .collect::<HashMap<_, _>>()
  }
}
