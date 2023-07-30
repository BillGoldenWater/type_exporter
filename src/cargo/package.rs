use std::path::PathBuf;

use serde::Deserialize;

use crate::cargo::target::Target;

#[derive(Debug, Clone, Deserialize)]
pub struct Package {
  pub name: String,
  pub manifest_path: PathBuf,
  pub targets: Vec<Target>,
}
