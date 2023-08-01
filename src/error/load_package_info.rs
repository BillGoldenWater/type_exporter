use crate::cargo::metadata::error::LoadMetadataError;

#[derive(Debug, thiserror::Error)]
pub enum LoadPackageInfoError {
  #[error("failed to load metadata: {0}")]
  LoadMetadata(#[from] LoadMetadataError),
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

pub type LoadPackageInfoResult<T> = Result<T, LoadPackageInfoError>;
