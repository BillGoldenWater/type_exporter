use std::fs;
use std::path::PathBuf;

use type_exporter::type_exporter::TypeExporter;

fn main() {
  let _ = env_logger::try_init();

  let output = PathBuf::from("./testExport");

  fs::create_dir_all(&output).expect("failed to create output dir");
  fs::remove_dir_all(&output).expect("failed to delete last output");
  fs::create_dir_all(&output).expect("failed to create output dir");

  TypeExporter::run(PathBuf::from("../ChaosDanmuTool/backend"), output).expect("failed to run");
}
