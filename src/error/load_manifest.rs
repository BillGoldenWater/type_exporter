use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoadManifestError {
  #[error("failed to get metadata: {0}")]
  Get(#[from] std::io::Error),
  #[error("cargo metadata exited with non zero status code, stderr: \n{0}")]
  CargoMetadataFailed(String),
  #[error("failed to parse metadata: {0}")]
  Parse(#[from] serde_json::Error),
}

pub type LoadManifestResult<T> = Result<T, LoadManifestError>;
