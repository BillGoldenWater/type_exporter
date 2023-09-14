use std::path::PathBuf;
use std::rc::Rc;

#[derive(Debug)]
pub struct PackageInfo {
  pub name: Rc<str>,
  pub root: PathBuf,
  pub entry: PathBuf,
}

impl PackageInfo {
  pub fn new(name: Rc<str>, root: PathBuf, entry: PathBuf) -> Self {
    Self { name, root, entry }
  }
}
