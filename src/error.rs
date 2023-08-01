use crate::cargo::metadata::error::LoadMetadataError;

pub mod process_manifest;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("failed to load metadata: {0}")]
  LoadMetadata(#[from] LoadMetadataError),
  #[error("failed to process manifest: {0}")]
  ProcessManifest(#[from] process_manifest::ProcessManifestError),
}

pub type TEResult<T> = Result<T, Error>;
