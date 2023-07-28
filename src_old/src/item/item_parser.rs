use log::debug;
use syn::{Item, ItemEnum, ItemStruct};

use crate::item::enum_info::EnumInfo;
use crate::item::item_info::ItemInfo;
use crate::item::struct_info::StructInfo;
use crate::use_path::{PathRs, UsePath};
use crate::{TEError, TEResult};

#[derive(Debug, Clone)]
pub struct ItemParser<'a> {
  uses: &'a Vec<UsePath<PathRs>>,
  local_items: &'a Vec<String>,
}

impl<'a> ItemParser<'a> {
  pub fn new(uses: &'a Vec<UsePath<PathRs>>, local_items: &'a Vec<String>) -> Self {
    Self { uses, local_items }
  }

  pub fn parse_item(&self, path: &UsePath<PathRs>, item: &Item) -> Option<TEResult<ItemInfo>> {
    match item {
      Item::Enum(it) => {
        debug!("parsing enum {}", it.ident);
        Some(self.parse_item_enum(path, it))
      }
      Item::Struct(it) => {
        debug!("parsing struct {}", it.ident);
        Some(self.parse_item_struct(path, it))
      }
      _ => None,
    }
  }

  pub fn parse_item_enum(
    &self,
    path: &UsePath<PathRs>,
    item_enum: &ItemEnum,
  ) -> TEResult<ItemInfo> {
    EnumInfo::parse_item_enum(path, self.uses, self.local_items, item_enum)
      .map(ItemInfo::from)
      .map_err(|err| TEError::ParseItemFailed(item_enum.ident.to_string(), err.into()))
  }

  pub fn parse_item_struct(
    &self,
    path: &UsePath<PathRs>,
    item_struct: &ItemStruct,
  ) -> TEResult<ItemInfo> {
    StructInfo::parse_item_struct(path, self.uses, self.local_items, item_struct)
      .map(ItemInfo::from)
      .map_err(|err| TEError::ParseItemFailed(item_struct.ident.to_string(), err.into()))
  }
}
