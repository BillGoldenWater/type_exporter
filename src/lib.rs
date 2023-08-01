#![warn(missing_debug_implementations)]

use std::path::PathBuf;

use itertools::Itertools;
use log::info;

use crate::cargo::metadata::Metadata;
use crate::config::Config;
use crate::error::load_package_info::{LoadPackageInfoError, LoadPackageInfoResult};
use crate::error::TEResult;
use crate::package::package_info::PackageInfo;

pub mod cargo;
pub mod config;
pub mod error;
pub mod package;

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
    let package_info = self.load_package_info()?;

    info!(
      "found target(s) {entries:?} in {name}({root:?})",
      entries = package_info.entries,
      name = package_info.name,
      root = package_info.root
    );

    Ok(())
  }

  fn load_package_info(&self) -> LoadPackageInfoResult<PackageInfo> {
    let metadata = Metadata::from_dir(&self.input)?;

    let available_packages = || {
      metadata
        .packages
        .iter()
        .map(|it| &it.name)
        .cloned()
        .collect_vec()
    };

    if metadata.packages.is_empty() {
      return Err(LoadPackageInfoError::NoPackage);
    }

    let package = if let Some(package_name) = &self.config.package_name {
      let package = metadata.packages.iter().find(|it| it.name.eq(package_name));

      if let Some(package) = package {
        package.clone()
      } else {
        return Err(
          LoadPackageInfoError::UnknownPackage {
            specified: package_name.clone(),
            available: available_packages(),
          },
        );
      }
    } else if metadata.packages.len() > 1 {
      return Err(LoadPackageInfoError::MultiPackage(available_packages()));
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

    Ok(PackageInfo::new(package.name, root, entries))
  }
}
