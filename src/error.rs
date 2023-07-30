pub mod load_manifest;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("failed to load manifest: {0}")]
  LoadManifest(#[from] load_manifest::LoadManifestError),
}

pub type TEResult<T> = Result<T, Error>;
