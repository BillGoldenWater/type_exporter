use std::path::PathBuf;

#[derive(Debug)]
pub struct PackageInfo {
  pub name: String,
  pub root: PathBuf,
  pub entries: Vec<PathBuf>,
}

impl PackageInfo {
  pub fn new(name: String, root: PathBuf, entries: Vec<PathBuf>) -> Self {
    Self {
      name,
      root,
      entries,
    }
  }
}
