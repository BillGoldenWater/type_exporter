/*
 * Copyright 2021-2023 Golden_Water
 * SPDX-License-Identifier: AGPL-3.0-only
 */

use std::collections::HashSet;
use std::str::FromStr;

use swc_core::ecma::ast;
use syn::__private::ToTokens;
use syn::{Field, GenericArgument, PathArguments, Type, TypePath};

use crate::item::attribute_info::AttributeInfo;
use crate::use_path::{PathRs, UsePath};
use crate::utils::ts_ast_utils::{create_keyword_type, create_type_array, create_type_ref};
use crate::{TEError, TEResult, TsAst};

#[derive(Debug, Clone)]
pub enum TypeInfo {
  Normal(UsePath<PathRs>),
  Option(Box<TypeInfo>),
  Vec(Box<TypeInfo>),
  Map(Box<TypeInfo>, Box<TypeInfo>),
  Box(Box<TypeInfo>),
  /// types from same file will also gets merged into this variant
  Custom(String),
  Bool,
  Number,
  BigInt,
  String,
}

impl TypeInfo {
  pub fn parse_type_path(
    path: &UsePath<PathRs>,
    uses: &Vec<UsePath<PathRs>>,
    local_items: &Vec<String>,
    field: &Field,
    attr: &AttributeInfo,
    type_path: &TypePath,
  ) -> TEResult<TypeInfo> {
    // region imported
    let path_segment = match UsePath::<PathRs>::resolve_type_from_uses(uses, &type_path.path) {
      Ok(path) => return Ok(TypeInfo::Normal(path)),
      Err(path_segment) => path_segment,
    };
    // endregion

    let type_name = path_segment.ident.to_string();
    // region primitives
    if let Ok(ty) = TypeInfo::from_str(&type_name) {
      return Ok(ty);
    }
    // endregion

    // region Option Vec etc.
    let generics = parse_path_generics(&path_segment.arguments);

    macro_rules! parse_with_generics {
      ($generics_args:expr, $num:expr, $name:ident<$($idx:expr),*) => {{
        check_generics_length(&type_name, $generics_args.len(), $num)?;
        Some(TypeInfo::$name(
          $(
            Box::from(Self::parse_type_path(
              path,
              uses,
              local_items,
              field,
              attr,
              &$generics_args[$idx],
            )?),
          )*
        ))
      }};
    }

    let type_info = match generics {
      Ok(generics) => match type_name.as_str() {
        "Option" => parse_with_generics!(generics, 1, Option < 0),
        "Vec" => parse_with_generics!(generics, 1, Vec < 0),
        "HashMap" => parse_with_generics!(generics, 2, Map < 0, 1),
        "Box" => parse_with_generics!(generics, 1, Box < 0),
        _ => None,
      },
      Err(_) => None,
    };

    if let Some(type_info) = type_info {
      return Ok(type_info);
    }
    // endregion

    if let Some(retype) = attr.retype.get() {
      Ok(TypeInfo::Custom(retype.clone()))
    } else if local_items.contains(&type_name) {
      Ok(TypeInfo::Normal(
        path.clone().with_name(type_name).with_local_use(true),
      ))
    } else {
      Err(TEError::UnknownType(
        field.ty.to_token_stream().to_string(),
        field.ty.clone(),
      ))
    }
  }

  pub fn to_ts_ast(&self) -> TsAst<ast::TsTypeAnn> {
    let mut dependencies = HashSet::new();

    let ts_type = match self {
      TypeInfo::Normal(rs_path) => {
        dependencies.insert(rs_path.clone());
        create_type_ref(&rs_path.name, None)
      }
      TypeInfo::Option(ty) => {
        let ty_ast = ty.to_ts_ast();
        dependencies.extend(ty_ast.dependencies);

        create_type_array(vec![
          ty_ast.ast.type_ann,
          Box::new(create_keyword_type(ast::TsKeywordTypeKind::TsNullKeyword)),
        ])
      }
      TypeInfo::Vec(ty) => {
        let ty_ast = ty.to_ts_ast();
        dependencies.extend(ty_ast.dependencies);

        create_type_ref("Array", Some(vec![ty_ast.ast.type_ann]))
      }
      TypeInfo::Map(ty_k, ty_v) => {
        let ty_k_ast = ty_k.to_ts_ast();
        let ty_v_ast = ty_v.to_ts_ast();
        dependencies.extend(ty_k_ast.dependencies);
        dependencies.extend(ty_v_ast.dependencies);

        create_type_ref(
          "Map",
          Some(vec![ty_k_ast.ast.type_ann, ty_v_ast.ast.type_ann]),
        )
      }
      TypeInfo::Box(ty) => return ty.to_ts_ast(),
      TypeInfo::Custom(ty) => create_type_ref(ty, None),
      TypeInfo::Bool => create_keyword_type(ast::TsKeywordTypeKind::TsBooleanKeyword),
      TypeInfo::Number => create_keyword_type(ast::TsKeywordTypeKind::TsNumberKeyword),
      TypeInfo::BigInt => create_keyword_type(ast::TsKeywordTypeKind::TsBigIntKeyword),
      TypeInfo::String => create_keyword_type(ast::TsKeywordTypeKind::TsStringKeyword),
    };

    TsAst {
      ast: ast::TsTypeAnn {
        span: Default::default(),
        type_ann: Box::new(ts_type),
      },
      dependencies,
    }
  }
}

impl FromStr for TypeInfo {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "bool" => Ok(Self::Bool),
      "u8" | "u16" | "u32" | "i8" | "i16" | "i32" | "f32" | "f64" => Ok(Self::Number),
      "u64" | "i64" | "usize" | "isize" => Ok(Self::BigInt),
      "String" | "char" => Ok(Self::String),
      _ => Err(()),
    }
  }
}

fn check_generics_length(name: &str, actual: usize, expected: usize) -> TEResult<()> {
  if actual != expected {
    Err(TEError::IncorrectGenericNumber(
      name.to_string(),
      expected,
      actual,
    ))
  } else {
    Ok(())
  }
}

fn parse_path_generics(path_arguments: &PathArguments) -> Result<Vec<TypePath>, ()> {
  let generics = match path_arguments {
    PathArguments::AngleBracketed(generics) => Ok(generics),
    _ => Err(()),
  }?;

  generics
    .args
    .iter()
    .map(|it| {
      if let GenericArgument::Type(Type::Path(type_path)) = it {
        return Ok(type_path.clone());
      }
      Err(())
    })
    .collect::<Result<Vec<_>, ()>>()
}
