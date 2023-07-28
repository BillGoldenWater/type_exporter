use std::path::Path;
use std::process::Command;

use crate::cargo::metadata::Metadata;
use crate::error::load_manifest::LoadManifestResult;

pub fn cargo_metadata(dir: impl AsRef<Path>) -> LoadManifestResult<Metadata> {
  Command::new("cargo")
    .args(["metadata", "--no-deps", "--format-version", "1"])
    .current_dir(dir)
    .output()
    .map(|it| serde_json::from_str(String::from_utf8_lossy(&it.stdout).as_ref()))?
    .map_err(Into::into)
}
