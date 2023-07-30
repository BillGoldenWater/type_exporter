use std::path::Path;
use std::process::Command;

use crate::cargo::metadata::Metadata;
use crate::error::load_manifest::{LoadManifestError, LoadManifestResult};

pub fn cargo_metadata(dir: impl AsRef<Path>) -> LoadManifestResult<Metadata> {
  let output = Command::new("cargo")
    .args(["metadata", "--no-deps", "--format-version", "1"])
    .current_dir(dir)
    .output()?;

  if !output.status.success() {
    return Err(LoadManifestError::CargoMetadataFailed(
      String::from_utf8_lossy(&output.stderr).to_string(),
    ));
  }

  serde_json::from_str(String::from_utf8_lossy(&output.stdout).as_ref()).map_err(Into::into)
}
