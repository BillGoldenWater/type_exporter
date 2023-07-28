use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Target {
  pub name: String,
  pub src_path: PathBuf,
}
