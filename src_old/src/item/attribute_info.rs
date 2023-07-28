/*
 * Copyright 2021-2023 Golden_Water
 * SPDX-License-Identifier: AGPL-3.0-only
 */

use std::str::FromStr;

use heck::{
  ToKebabCase, ToLowerCamelCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase,
  ToUpperCamelCase,
};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Attribute, Ident, LitStr, Meta, Token};

use crate::{TEError, TEResult};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AttributeInfo {
  pub entry: AttributeInfoItem<String>,

  pub retype: AttributeInfoItem<String>,
  pub rename: AttributeInfoItem<String>,
  pub rename_all: AttributeInfoItem<RenameAll>,
  pub tag: AttributeInfoItem<String>,
  pub tag_content: AttributeInfoItem<String>,
  pub skip: AttributeInfoItem<String>,
  pub skip_serializing: AttributeInfoItem<String>,
}

impl AttributeInfo {
  pub fn is_skipped(&self) -> bool {
    self.skip.is_set() || self.skip_serializing.is_set()
  }

  pub fn is_entry(&self) -> bool {
    self.entry.is_set()
  }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum AttributeInfoItem<T> {
  #[default]
  Unset,
  SetEmpty,
  Set(T),
}

impl<T> AttributeInfoItem<T> {
  pub fn is_set(&self) -> bool {
    match self {
      AttributeInfoItem::Unset => false,
      AttributeInfoItem::SetEmpty | AttributeInfoItem::Set(..) => true,
    }
  }

  pub fn get(&self) -> Option<&T> {
    match self {
      AttributeInfoItem::Unset | AttributeInfoItem::SetEmpty => None,
      AttributeInfoItem::Set(v) => Some(v),
    }
  }

  pub fn as_ref(&self) -> AttributeInfoItem<&T> {
    match self {
      Self::Unset => AttributeInfoItem::Unset,
      Self::SetEmpty => AttributeInfoItem::SetEmpty,
      Self::Set(v) => AttributeInfoItem::Set(v),
    }
  }
}

impl<T> From<Option<T>> for AttributeInfoItem<T> {
  fn from(value: Option<T>) -> Self {
    match value {
      None => Self::SetEmpty,
      Some(value) => Self::Set(value),
    }
  }
}

impl<T> From<AttributeInfoItem<T>> for Option<T> {
  fn from(value: AttributeInfoItem<T>) -> Self {
    match value {
      AttributeInfoItem::Unset | AttributeInfoItem::SetEmpty => None,
      AttributeInfoItem::Set(v) => Some(v),
    }
  }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum RenameAll {
  LowerCase,
  UpperCase,
  PascalCase,
  #[default]
  CamelCase,
  SnakeCase,
  ScreamingSnakeCase,
  KebabCase,
  ScreamingKebabCase,
}

impl RenameAll {
  pub fn do_convert<S: AsRef<str>>(&self, string: S) -> String {
    let string = string.as_ref();
    match self {
      RenameAll::LowerCase => string.to_lowercase(),
      RenameAll::UpperCase => string.to_uppercase(),
      RenameAll::PascalCase => string.to_upper_camel_case(),
      RenameAll::CamelCase => string.to_lower_camel_case(),
      RenameAll::SnakeCase => string.to_snake_case(),
      RenameAll::ScreamingSnakeCase => string.to_shouty_snake_case(),
      RenameAll::KebabCase => string.to_kebab_case(),
      RenameAll::ScreamingKebabCase => string.to_shouty_kebab_case(),
    }
  }
}

impl FromStr for RenameAll {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "lowercase" => Ok(Self::LowerCase),
      "UPPERCASE" => Ok(Self::UpperCase),
      "PascalCase" => Ok(Self::PascalCase),
      "camelCase" => Ok(Self::CamelCase),
      "snake_case" => Ok(Self::SnakeCase),
      "SCREAMING_SNAKE_CASE" => Ok(Self::ScreamingSnakeCase),
      "kebab-case" => Ok(Self::KebabCase),
      "SCREAMING-KEBAB-CASE" => Ok(Self::ScreamingKebabCase),
      _ => Err(()),
    }
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Arg {
  ident: Ident,
  value: Option<LitStr>,
}

impl Parse for Arg {
  fn parse(parse_stream: ParseStream) -> syn::Result<Self> {
    let ident = parse_stream.parse::<Ident>()?;

    if parse_stream.parse::<Token![=]>().is_ok() {
      let value = parse_stream.parse::<LitStr>()?;

      Ok(Self {
        ident,
        value: Some(value),
      })
    } else {
      Ok(Self { ident, value: None })
    }
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AttributeArgs {
  args: Punctuated<Arg, Token![,]>,
}

impl Parse for AttributeArgs {
  fn parse(parse_stream: ParseStream) -> syn::Result<Self> {
    Ok(Self {
      args: parse_stream.parse_terminated(Arg::parse, Token![,])?,
    })
  }
}

pub fn parse_attributes(attrs: &[Attribute]) -> TEResult<AttributeInfo> {
  let mut result = AttributeInfo::default();

  for attr in attrs {
    if let Meta::List(meta_list) = &attr.meta {
      macro_rules! match_apply {
        ($key:expr, $value:expr, $result:expr; $($match_str:literal => $field:ident,)*) => {
          match $key {
            $($match_str => $result.$field = $value,)*
            _ => {}
          }
        };
      }

      match meta_list
        .path
        .segments
        .last()
        .unwrap()
        .ident
        .to_string()
        .as_str()
      {
        "te" => {
          let args = syn::parse2::<AttributeArgs>(meta_list.tokens.clone())?;
          for arg in args.args {
            let key = arg.ident.to_string();
            let value = arg.value.map(|it| it.value());

            match_apply! { key.as_str(), value.into(), result;
              "entry" => entry,

              "retype" => retype,
              "rename" => rename,
            }
          }
        }
        "serde" => {
          let args = syn::parse2::<AttributeArgs>(meta_list.tokens.clone())?;
          for arg in args.args {
            let key = arg.ident.to_string();
            let value = arg.value.map(|it| it.value());

            if key.eq("rename_all") {
              if let Some(ref value) = value {
                let rename_all = RenameAll::from_str(value)
                  .map_err(|_| TEError::UnknownValueOfRenameAll(value.clone()))?;

                result.rename_all = AttributeInfoItem::Set(rename_all);
              } else {
                result.rename_all = AttributeInfoItem::SetEmpty;
              }
            }

            match_apply! { key.as_str(), value.into(), result;
              "rename" => rename,
              "tag" => tag,
              "content" => tag_content,
              "skip" => skip,
              "skip_serializing" => skip_serializing,
            }
          }
        }
        _ => {}
      }
    }
  }

  Ok(result)
}
