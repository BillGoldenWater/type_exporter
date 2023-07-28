use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoadManifestError {
  #[error("failed to get metadata: {0}")]
  Get(#[from] std::io::Error),
  #[error("failed to parse metadata: {0}")]
  Parse(#[from] serde_json::Error),
}

pub type LoadManifestResult<T> = Result<T, LoadManifestError>;
