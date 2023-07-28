use thiserror::Error;

#[derive(Debug, Error)]
pub enum InitializeError {
  #[error("failed to read config: {0}")]
  ReadConfig(#[from] std::io::Error),
  #[error("failed to parse config: {0}")]
  ParseConfig(#[from] toml::de::Error),
}

pub type InitializeResult<T> = Result<T, InitializeError>;
