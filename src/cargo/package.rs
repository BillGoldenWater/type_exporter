use serde::Deserialize;

use crate::cargo::target::Target;

#[derive(Debug, Clone, Deserialize)]
pub struct Package {
  pub name: String,
  pub targets: Vec<Target>,
}
