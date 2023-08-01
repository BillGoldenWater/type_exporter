pub mod load_package_info;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("failed to load package info: {0}")]
  LoadPackageInfo(#[from] load_package_info::LoadPackageInfoError),
}

pub type TEResult<T> = Result<T, Error>;
