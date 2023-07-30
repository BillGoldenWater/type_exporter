#![warn(missing_debug_implementations)]

use std::path::PathBuf;

use itertools::Itertools;
use log::info;

use crate::config::Config;
use crate::error::process_manifest::ProcessManifestError;
use crate::error::TEResult;
use crate::package::Package;
use crate::utils::cargo_utils::cargo_metadata;

pub mod cargo;
pub mod config;
pub mod error;
pub mod package;
pub mod utils;

#[derive(Debug)]
pub struct TypeExporter {
  input: PathBuf,
  output: PathBuf,
  config: Config,
}

impl TypeExporter {
  pub fn new(input_path: PathBuf, output_path: PathBuf, config: Config) -> Self {
    Self {
      input: input_path,
      output: output_path,
      config,
    }
  }

  pub fn export(&self) -> TEResult<()> {
    let package = self.process_manifest()?;

    info!(
      "found target(s) {entries:?} in {name}({root:?})",
      entries = package.entries,
      name = package.name,
      root = package.root
    );

    Ok(())
  }

  fn process_manifest(&self) -> TEResult<Package> {
    let metadata = cargo_metadata(&self.input)?;

    let available_packages = || {
      metadata
        .packages
        .iter()
        .map(|it| &it.name)
        .cloned()
        .collect_vec()
    };

    if metadata.packages.is_empty() {
      return Err(ProcessManifestError::NoPackage.into());
    }

    let package = if let Some(package_name) = &self.config.package_name {
      let package = metadata.packages.iter().find(|it| it.name.eq(package_name));

      if let Some(package) = package {
        package.clone()
      } else {
        return Err(
          ProcessManifestError::UnknownPackage {
            specified: package_name.clone(),
            available: available_packages(),
          }
          .into(),
        );
      }
    } else if metadata.packages.len() > 1 {
      return Err(ProcessManifestError::MultiPackage(available_packages()).into());
    } else {
      metadata.packages[0].clone()
    };

    let root = package
      .manifest_path
      .parent()
      .expect("expect has parent")
      .to_path_buf();

    let entries = package
      .targets
      .into_iter()
      .map(|it| {
        it.src_path
          .strip_prefix(&root)
          .expect("expect start with root")
          .strip_prefix("src")
          .expect("expect inside src folder")
          .to_path_buf()
      })
      .collect();

    Ok(Package::new(package.name, root, entries))
  }
}
