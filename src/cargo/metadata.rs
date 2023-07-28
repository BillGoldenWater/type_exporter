use serde::Deserialize;

use crate::cargo::package::Package;

#[derive(Debug, Clone, Deserialize)]
pub struct Metadata {
  pub packages: Vec<Package>,
}
