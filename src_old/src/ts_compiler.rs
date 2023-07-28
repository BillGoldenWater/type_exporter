use std::sync::Arc;

use swc::config::SourceMapsConfig;
use swc::Compiler;
use swc_common::collections::AHashMap;
use swc_common::{FilePathMapping, Globals, SourceMap, GLOBALS};
use swc_core::ecma::ast;
use swc_core::ecma::ast::ModuleItem;

pub struct TsCompiler {
  compiler: Compiler,
  globals: Globals,
}

impl Default for TsCompiler {
  fn default() -> Self {
    let source_map = SourceMap::new(FilePathMapping::new(vec![]));
    let source_map = Arc::new(source_map);

    let compiler = Compiler::new(source_map);

    Self {
      compiler,
      globals: Globals::new(),
    }
  }
}

impl TsCompiler {
  pub fn compile(&self, content: Vec<ModuleItem>) -> String {
    let program = ast::Program::Module(ast::Module {
      span: Default::default(),
      body: content,
      shebang: None,
    });

    let mut output = String::new();
    GLOBALS.set(&self.globals, || {
      output = self
        .compiler
        .print(
          &program,
          None,
          None,
          false,
          ast::EsVersion::Es2021,
          SourceMapsConfig::Bool(false),
          &AHashMap::default(),
          None,
          false,
          None,
          false,
          false,
        )
        .expect("Failed to compile")
        .code;
    });

    output
  }
}
