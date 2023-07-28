use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
  pub overwrite: Option<Vec<GlobalTypeOverwriteItem>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GlobalTypeOverwriteItem {
  pub from: String,
  pub to: String,
}
