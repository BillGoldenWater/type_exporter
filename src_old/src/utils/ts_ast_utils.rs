/*
 * Copyright 2021-2023 Golden_Water
 * SPDX-License-Identifier: AGPL-3.0-only
 */

use crate::use_path::{PathRs, UsePath};
use std::path::PathBuf;
use swc_core::ecma::ast;
use swc_core::ecma::ast::Str;
use swc_core::ecma::atoms::JsWord;

pub fn create_type_array(types: Vec<Box<ast::TsType>>) -> ast::TsType {
  ast::TsType::TsUnionOrIntersectionType(ast::TsUnionOrIntersectionType::TsUnionType(
    ast::TsUnionType {
      span: Default::default(),
      types,
    },
  ))
}

pub fn create_type_ref<T: AsRef<str>>(ty: T, params: Option<Vec<Box<ast::TsType>>>) -> ast::TsType {
  ast::TsType::TsTypeRef(ast::TsTypeRef {
    span: Default::default(),
    type_name: ast::TsEntityName::Ident(ast::Ident::new(
      JsWord::from(ty.as_ref()),
      Default::default(),
    )),
    type_params: params.map(|it| {
      Box::new(ast::TsTypeParamInstantiation {
        span: Default::default(),
        params: it,
      })
    }),
  })
}

pub fn create_keyword_type(type_kind: ast::TsKeywordTypeKind) -> ast::TsType {
  ast::TsType::TsKeywordType(ast::TsKeywordType {
    span: Default::default(),
    kind: type_kind,
  })
}

pub fn create_ident<S: AsRef<str>>(ident: S) -> ast::Ident {
  ast::Ident::new(JsWord::from(ident.as_ref()), Default::default())
}

pub fn create_expr_ident<S: AsRef<str>>(ident: S) -> ast::Expr {
  ast::Expr::Ident(create_ident(ident))
}

pub fn create_expr_str<S: AsRef<str>>(ident: S) -> ast::Expr {
  ast::Expr::Lit(ast::Lit::Str(Str::from(ident.as_ref())))
}

pub fn create_module_decl_item(decl: ast::Decl) -> ast::ModuleItem {
  ast::ModuleItem::ModuleDecl(ast::ModuleDecl::ExportDecl(ast::ExportDecl {
    span: Default::default(),
    decl,
  }))
}

pub fn create_type_lit(members: Vec<ast::TsTypeElement>) -> ast::TsType {
  ast::TsType::TsTypeLit(ast::TsTypeLit {
    span: Default::default(),
    members,
  })
}

pub fn create_property_type_element(
  ident: ast::Expr,
  type_ann: ast::TsTypeAnn,
) -> ast::TsTypeElement {
  ast::TsTypeElement::TsPropertySignature(ast::TsPropertySignature {
    span: Default::default(),
    readonly: false,
    key: Box::new(ident),
    computed: false,
    optional: false,
    init: None,
    params: vec![],
    type_ann: Some(Box::new(type_ann)),
    type_params: None,
  })
}

pub fn type_to_type_ann(ts_type: ast::TsType) -> ast::TsTypeAnn {
  ast::TsTypeAnn {
    span: Default::default(),
    type_ann: Box::new(ts_type),
  }
}

pub fn create_str_lit_type<S: AsRef<str>>(string: S) -> ast::TsType {
  ast::TsType::TsLitType(ast::TsLitType {
    span: Default::default(),
    lit: ast::TsLit::Str(ast::Str::from(string.as_ref())),
  })
}

pub fn create_type_alias_decl(id: ast::Ident, ts_type: ast::TsType) -> ast::Decl {
  ast::Decl::TsTypeAlias(Box::new(ast::TsTypeAliasDecl {
    span: Default::default(),
    declare: true,
    id,
    type_params: None,
    type_ann: Box::new(ts_type),
  }))
}

pub fn create_import(path: &UsePath<PathRs>) -> ast::ModuleItem {
  ast::ModuleItem::ModuleDecl(ast::ModuleDecl::Import(ast::ImportDecl {
    span: Default::default(),
    specifiers: vec![ast::ImportSpecifier::Named(ast::ImportNamedSpecifier {
      span: Default::default(),
      local: create_ident(&path.name),
      imported: path
        .actual_name
        .as_ref()
        .map(|it| ast::ModuleExportName::Ident(create_ident(it))),
      is_type_only: false,
    })],
    src: Box::new(ast::Str::from(
      PathBuf::from(path.to_fs()).to_string_lossy(),
    )),
    type_only: true,
    asserts: None,
  }))
}
