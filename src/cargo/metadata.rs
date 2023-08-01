use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use crate::cargo::metadata::error::{LoadMetadataError, LoadMetadataResult};
use crate::cargo::package::Package;

pub mod error;

#[derive(Debug, Clone, Deserialize)]
pub struct Metadata {
  pub packages: Vec<Package>,
}

impl Metadata {
  pub fn from_dir(dir: impl AsRef<Path>) -> LoadMetadataResult<Self> {
    let output = Command::new("cargo")
      .args(["metadata", "--no-deps", "--format-version", "1"])
      .current_dir(dir)
      .output()?;

    if !output.status.success() {
      return Err(LoadMetadataError::CargoMetadataFailed(
        String::from_utf8_lossy(&output.stderr).to_string(),
      ));
    }

    serde_json::from_str(String::from_utf8_lossy(&output.stdout).as_ref()).map_err(Into::into)
  }
}
