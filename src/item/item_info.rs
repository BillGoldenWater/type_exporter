use crate::item::enum_info::EnumInfo;
use crate::item::struct_info::StructInfo;

#[derive(Debug, Clone)]
pub enum ItemInfo {
  Struct { processed: bool, item: StructInfo },
  Enum { processed: bool, item: EnumInfo },
}

impl ItemInfo {
  pub fn get_name(&self) -> &str {
    match self {
      ItemInfo::Struct { item, .. } => item.name.as_str(),
      ItemInfo::Enum { item, .. } => item.name.as_str(),
    }
  }
}

impl From<EnumInfo> for ItemInfo {
  fn from(value: EnumInfo) -> Self {
    Self::Enum {
      item: value,
      processed: false,
    }
  }
}

impl From<StructInfo> for ItemInfo {
  fn from(value: StructInfo) -> Self {
    Self::Struct {
      item: value,
      processed: false,
    }
  }
}
