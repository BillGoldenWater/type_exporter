use std::path::PathBuf;

pub struct Package {
  pub name: String,
  pub root: PathBuf,
  pub entries: Vec<PathBuf>,
}

impl Package {
  pub fn new(name: String, root: PathBuf, entries: Vec<PathBuf>) -> Self {
    Self {
      name,
      root,
      entries,
    }
  }
}
