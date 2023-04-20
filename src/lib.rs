use std::collections::HashSet;

use log::error;

pub use type_exporter_macro::*;

use crate::use_path::{PathRs, UsePath};

pub mod item;
pub mod ts_compiler;
pub mod type_exporter;
pub mod use_path;
pub mod utils;

#[derive(thiserror::Error, Debug)]
pub enum TEError {
  #[error("the root path isn't the root of a valid cargo project")]
  InvalidCargoProjectRoot,
  #[error("unknown type: {0}, detail: {1:?}")]
  UnknownType(String, syn::Type),
  #[error("incorrect generic number for {0}, expected: {1}, actually: {2}")]
  IncorrectGenericNumber(String, usize, usize),
  #[error("failed to parse item {0}: {1:?}")]
  ParseItemFailed(String, Box<TEError>),
  #[error("unknown value of rename_all: {0}")]
  UnknownValueOfRenameAll(String),
  #[error("failed to do read/write operation: {0}")]
  Io(#[from] std::io::Error),
  #[error("failed to parse: {0}")]
  Syn(#[from] syn::Error),
  #[error("failed to list all files: {0}")]
  WalkDir(#[from] walkdir::Error),
}

pub type TEResult<T> = Result<T, TEError>;

#[derive(Debug)]
pub struct TsAst<T> {
  ast: T,
  dependencies: HashSet<UsePath<PathRs>>,
}
