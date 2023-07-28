/*
 * Copyright 2021-2023 Golden_Water
 * SPDX-License-Identifier: AGPL-3.0-only
 */

use crate::item::attribute_info::{AttributeInfo, RenameAll};

pub mod ts_ast_utils;

pub fn rename_name(
  attr: &AttributeInfo,
  rename_all: Option<&RenameAll>,
  name: Option<&String>,
) -> Option<String> {
  if let Some(rename) = attr.rename.clone().into() {
    return Some(rename);
  }

  name.cloned().map(|name| {
    if let Some(rename_all) = rename_all {
      rename_all.do_convert(name)
    } else {
      name
    }
  })
}
