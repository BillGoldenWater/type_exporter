pub mod load_manifest;
pub mod process_manifest;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("failed to load manifest: {0}")]
  LoadManifest(#[from] load_manifest::LoadManifestError),
  #[error("failed to process manifest: {0}")]
  ProcessManifest(#[from] process_manifest::ProcessManifestError),
}

pub type TEResult<T> = Result<T, Error>;
