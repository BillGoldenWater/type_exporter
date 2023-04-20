use std::path::PathBuf;

use type_exporter::type_exporter::TypeExporter;

fn main() {
  let _ = env_logger::try_init();

  let args: Args = argh::from_env();

  TypeExporter::run(PathBuf::from(args.input), PathBuf::from(args.output)).expect("failed to run");
}

#[derive(argh::FromArgs)]
#[argh(description = "a tool for generate typescript definition from rust")]
struct Args {
  /// path to input cargo project
  #[argh(option, short = 'i')]
  input: String,
  /// path to output
  #[argh(option, short = 'o')]
  output: String,
}
