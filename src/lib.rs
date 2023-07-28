#![warn(missing_debug_implementations)]

use std::path::PathBuf;

use crate::config::Config;
use crate::error::initialize::InitializeResult;
use crate::error::TEResult;

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
  pub fn new(
    input_path: PathBuf,
    output_path: PathBuf,
    config_path: Option<PathBuf>,
  ) -> InitializeResult<Self> {
    let config = config_path
      .map(std::fs::read_to_string)
      .transpose()?
      .map(|config_str| toml::from_str::<Config>(&config_str))
      .transpose()?;

    Ok(Self {
      input: input_path,
      output: output_path,
      config: config.unwrap_or_default(),
    })
  }

  pub fn export(&mut self) -> TEResult<()> {
    Ok(())
  }
}
