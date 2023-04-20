/*
 * Copyright 2021-2023 Golden_Water
 * SPDX-License-Identifier: AGPL-3.0-only
 */

use swc_core::ecma::ast;
use syn::__private::ToTokens;
use syn::{Fields, Type};

use crate::item::attribute_info::{parse_attributes, AttributeInfo, RenameAll};
use crate::item::type_info::TypeInfo;
use crate::use_path::{PathRs, UsePath};
use crate::utils::rename_name;
use crate::utils::ts_ast_utils::{create_expr_ident, create_ident, create_property_type_element};
use crate::{TEError, TEResult, TsAst};

#[derive(Debug, Clone)]
pub struct FieldInfo {
  pub name: Option<String>,
  pub ty: TypeInfo,
  pub attr: AttributeInfo,
}

impl FieldInfo {
  pub fn parse_fields(
    path: &UsePath<PathRs>,
    uses: &Vec<UsePath<PathRs>>,
    local_items: &Vec<String>,
    fields: &Fields,
  ) -> TEResult<Vec<Self>> {
    fields
      .into_iter()
      .map(|it| match &it.ty {
        Type::Path(type_path) => {
          let attr = parse_attributes(&it.attrs)?;

          Ok(FieldInfo {
            name: it.ident.as_ref().map(|it| it.to_string()),
            ty: TypeInfo::parse_type_path(path, uses, local_items, it, &attr, type_path)?,
            attr,
          })
        }
        _ => Err(TEError::UnknownType(
          it.ty.to_token_stream().to_string(),
          it.ty.clone(),
        )),
      })
      .collect::<TEResult<Vec<FieldInfo>>>()
  }

  pub fn to_ts_ast_named(
    &self,
    rename_all: Option<&RenameAll>,
  ) -> Option<TsAst<ast::TsTypeElement>> {
    let name = rename_name(&self.attr, rename_all, self.name.as_ref())?;

    let ty_ast = self.ty.to_ts_ast();
    let ts_ast = TsAst {
      ast: create_property_type_element(create_expr_ident(name), ty_ast.ast),
      dependencies: ty_ast.dependencies,
    };

    Some(ts_ast)
  }

  pub fn to_ts_ast_unnamed(&self, rename_all: Option<&RenameAll>) -> TsAst<ast::TsTupleElement> {
    let label = rename_name(&self.attr, rename_all, self.name.as_ref()).map(|it| {
      ast::Pat::Ident(ast::BindingIdent {
        id: create_ident(it),
        type_ann: None,
      })
    });

    let ty_ast = self.ty.to_ts_ast();
    TsAst {
      ast: ast::TsTupleElement {
        span: Default::default(),
        label,
        ty: ty_ast.ast.type_ann,
      },
      dependencies: Default::default(),
    }
  }
}
