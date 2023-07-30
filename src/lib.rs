#![warn(missing_debug_implementations)]

use std::path::PathBuf;

use crate::config::Config;
use crate::error::TEResult;
use crate::utils::cargo_utils::cargo_metadata;

pub mod cargo;
pub mod config;
pub mod error;
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
    let metadata = cargo_metadata(&self.input)?;

    Ok(())
  }
}
