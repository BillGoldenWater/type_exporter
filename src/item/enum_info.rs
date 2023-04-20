/*
 * Copyright 2021-2023 Golden_Water
 * SPDX-License-Identifier: AGPL-3.0-only
 */

use std::collections::HashSet;

use log::error;
use swc_core::ecma::ast;
use syn::{ItemEnum, Variant};

use crate::item::attribute_info::{parse_attributes, AttributeInfo, RenameAll};
use crate::item::field_info::FieldInfo;
use crate::item::struct_info::StructInfo;
use crate::item::type_info::TypeInfo;
use crate::use_path::{PathRs, UsePath};
use crate::utils::rename_name;
use crate::utils::ts_ast_utils::{
  create_expr_ident, create_expr_str, create_ident, create_module_decl_item,
  create_property_type_element, create_str_lit_type, create_type_alias_decl, create_type_array,
  create_type_lit, create_type_ref, type_to_type_ann,
};
use crate::{TEResult, TsAst};

#[derive(Debug, Clone)]
pub struct EnumInfo {
  pub name: String,
  pub attr: AttributeInfo,
  pub variants: Vec<VariantInfo>,
}

impl EnumInfo {
  pub fn parse_item_enum(
    path: &UsePath<PathRs>,
    uses: &Vec<UsePath<PathRs>>,
    local_items: &Vec<String>,
    item_enum: &ItemEnum,
  ) -> TEResult<EnumInfo> {
    let variants = item_enum
      .variants
      .iter()
      .map(|it| VariantInfo::parse_variant(path, uses, local_items, it))
      .collect::<TEResult<Vec<_>>>()?;

    Ok(EnumInfo {
      name: item_enum.ident.to_string(),
      attr: parse_attributes(&item_enum.attrs)?,
      variants,
    })
  }

  pub fn to_ts_ast(&self) -> TsAst<Vec<ast::ModuleItem>> {
    let name = rename_name(&self.attr, None, Some(&self.name)).unwrap();

    let mut dependencies = HashSet::new();
    let mut variants = vec![];
    let mut variant_types = vec![];

    for variant_info in &self.variants {
      let ts_ast = variant_info.to_ts_ast(
        &name,
        self.attr.rename_all.as_ref().into(),
        self.attr.tag.as_ref().into(),
        self.attr.tag_content.as_ref().into(),
      );

      dependencies.extend(ts_ast.dependencies);
      variants.push(Box::new(ts_ast.ast.0));
      if let Some(ty) = ts_ast.ast.1 {
        variant_types.push(ty)
      }
    }

    let module_item = create_module_decl_item(create_type_alias_decl(
      create_ident(&name),
      create_type_array(variants),
    ));

    variant_types.push(module_item);

    TsAst {
      ast: variant_types,
      dependencies,
    }
  }
}

#[derive(Debug, Clone)]
pub struct VariantInfo {
  pub name: String,
  pub attr: AttributeInfo,
  pub fields: Vec<FieldInfo>,
}

impl VariantInfo {
  pub fn parse_variant(
    path: &UsePath<PathRs>,
    uses: &Vec<UsePath<PathRs>>,
    types: &Vec<String>,
    variant: &Variant,
  ) -> TEResult<Self> {
    Ok(Self {
      name: variant.ident.to_string(),
      attr: parse_attributes(&variant.attrs)?,
      fields: FieldInfo::parse_fields(path, uses, types, &variant.fields)?,
    })
  }

  pub fn is_normal_variant(&self) -> bool {
    if self.is_unit_variant() {
      return false;
    }
    if self.fields[0].name.is_none() {
      return false;
    }

    true
  }

  pub fn is_tuple_variant(&self) -> bool {
    if self.is_unit_variant() {
      return false;
    }
    if self.fields[0].name.is_some() {
      return false;
    }

    true
  }

  pub fn is_unit_variant(&self) -> bool {
    if self.fields.is_empty() {
      return true;
    }
    if self.fields.len() == 1 && self.fields[0].attr.is_skipped() {
      return true;
    }
    false
  }

  pub fn to_ts_ast<Name: AsRef<str>>(
    &self,
    enum_name: Name,
    rename_all: Option<&RenameAll>,
    tag: Option<&String>,
    content: Option<&String>,
  ) -> TsAst<(ast::TsType, Option<ast::ModuleItem>)> {
    let name = rename_name(&self.attr, rename_all, Some(&self.name)).unwrap();
    let variant_type_name = format!("{}_{}", enum_name.as_ref(), name);

    let mut dependencies = HashSet::new();

    fn to_struct_ast(
      this: &VariantInfo,
      name: &str,
      mut fields_prepend: Vec<FieldInfo>,
    ) -> TsAst<ast::ModuleItem> {
      fields_prepend.extend(this.fields.clone());
      StructInfo {
        name: name.to_string(),
        fields: fields_prepend,
        attr: AttributeInfo {
          rename: None.into(),
          ..this.attr.clone()
        },
      }
      .to_ts_ast()
    }

    let ast = if let Some(tag) = tag.cloned() {
      if self.is_unit_variant() {
        (
          create_type_lit(vec![create_property_type_element(
            create_expr_ident(tag),
            type_to_type_ann(create_str_lit_type(name)),
          )]),
          None,
        )
      } else if let Some(content) = content.cloned() {
        // region adjacently tagged
        let ts_ast = to_struct_ast(self, &variant_type_name, vec![]);
        dependencies = ts_ast.dependencies;

        let type_in_enum_define = create_type_lit(vec![
          create_property_type_element(
            create_expr_ident(tag),
            type_to_type_ann(create_str_lit_type(name)),
          ),
          create_property_type_element(
            create_expr_ident(content),
            type_to_type_ann(create_type_ref(variant_type_name, None)),
          ),
        ]);
        (type_in_enum_define, Some(ts_ast.ast))
        // endregion
      } else {
        // region internally tagged
        if self.is_tuple_variant() {
          error!(
            "internally tagged with tuple variant isn't expected, this will produce a wrong result"
          );
        }

        let ts_ast = to_struct_ast(
          self,
          &variant_type_name,
          vec![FieldInfo {
            name: self.fields[0].name.clone(),
            ty: TypeInfo::Custom(format!("\"{name}\"")),
            attr: AttributeInfo {
              rename: Some(tag).into(),
              ..Default::default()
            },
          }],
        );
        dependencies = ts_ast.dependencies;

        (create_type_ref(variant_type_name, None), Some(ts_ast.ast))
        // endregion
      }
    } else {
      // region externally tagged
      if self.is_unit_variant() {
        (create_str_lit_type(name), None)
      } else {
        let ts_ast = to_struct_ast(self, &variant_type_name, vec![]);
        dependencies = ts_ast.dependencies;

        let type_in_enum_define = create_type_lit(vec![create_property_type_element(
          create_expr_str(name),
          type_to_type_ann(create_type_ref(variant_type_name, None)),
        )]);
        (type_in_enum_define, Some(ts_ast.ast))
      }
      // endregion
    };

    TsAst { ast, dependencies }
  }
}
