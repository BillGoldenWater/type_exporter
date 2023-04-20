/*
 * Copyright 2021-2023 Golden_Water
 * SPDX-License-Identifier: AGPL-3.0-only
 */

use std::collections::HashSet;

use swc_core::ecma::ast;
use syn::ItemStruct;

use crate::item::attribute_info::{parse_attributes, AttributeInfo, RenameAll};
use crate::item::field_info::FieldInfo;
use crate::use_path::{PathRs, UsePath};
use crate::utils::rename_name;
use crate::utils::ts_ast_utils::{
  create_ident, create_keyword_type, create_module_decl_item, create_type_alias_decl,
  create_type_lit,
};
use crate::{TEResult, TsAst};

#[derive(Debug, Clone)]
pub struct StructInfo {
  pub name: String,
  pub fields: Vec<FieldInfo>,
  pub attr: AttributeInfo,
}

impl StructInfo {
  pub fn parse_item_struct(
    path: &UsePath<PathRs>,
    uses: &Vec<UsePath<PathRs>>,
    local_items: &Vec<String>,
    item_struct: &ItemStruct,
  ) -> TEResult<Self> {
    Ok(Self {
      name: item_struct.ident.to_string(),
      fields: FieldInfo::parse_fields(path, uses, local_items, &item_struct.fields)?,
      attr: parse_attributes(&item_struct.attrs)?,
    })
  }

  pub fn is_normal_struct(&self) -> bool {
    if self.is_unit_struct() {
      return false;
    }
    if self.fields[0].name.is_none() {
      return false;
    }

    true
  }

  pub fn is_tuple_struct(&self) -> bool {
    if self.is_unit_struct() {
      return false;
    }
    if self.fields[0].name.is_some() {
      return false;
    }

    true
  }

  pub fn is_unit_struct(&self) -> bool {
    if self.fields.is_empty() {
      return true;
    }
    false
  }

  pub fn to_ts_ast(&self) -> TsAst<ast::ModuleItem> {
    let mut dependencies = HashSet::new();

    let rename_all: Option<RenameAll> = self.attr.rename_all.clone().into();

    let type_ann = if self.is_normal_struct() {
      let mut members = vec![];

      for field in &self.fields {
        if field.attr.is_skipped() {
          continue;
        }
        let ts_ast = field
          .to_ts_ast_named(rename_all.as_ref())
          .expect("unexpect unnamed field inside a normal struct");

        members.push(ts_ast.ast);
        dependencies.extend(ts_ast.dependencies);
      }

      create_type_lit(members)
    } else if self.is_tuple_struct() {
      if self.fields.len() > 1 {
        let mut elem_types = vec![];

        for field in &self.fields {
          if field.attr.is_skipped() {
            continue;
          }
          let ts_ast = field.to_ts_ast_unnamed(rename_all.as_ref());

          elem_types.push(ts_ast.ast);
          dependencies.extend(ts_ast.dependencies);
        }

        ast::TsType::TsTupleType(ast::TsTupleType {
          span: Default::default(),
          elem_types,
        })
      } else {
        let ts_ast = self.fields[0].ty.to_ts_ast();
        dependencies.extend(ts_ast.dependencies);
        *ts_ast.ast.type_ann
      }
    } else {
      create_keyword_type(ast::TsKeywordTypeKind::TsNullKeyword)
    };

    let decl = create_type_alias_decl(
      create_ident(&rename_name(&self.attr, None.into(), Some(&self.name)).unwrap()),
      type_ann,
    );

    TsAst {
      ast: create_module_decl_item(decl),
      dependencies,
    }
  }
}
