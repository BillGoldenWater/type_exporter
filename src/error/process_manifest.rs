#[derive(Debug, thiserror::Error)]
pub enum ProcessManifestError {
  #[error("no package in the input path")]
  NoPackage,
  #[error(
    "more than one package in the input path, you need specify one, available packages: {0:?}"
  )]
  MultiPackage(Vec<String>),
  #[error("specified package {specified:?} not exists, available packages: {available:?}")]
  UnknownPackage {
    specified: String,
    available: Vec<String>,
  },
}

pub type ProcessManifestResult<T> = Result<T, ProcessManifestError>;
