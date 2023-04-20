use std::ffi::OsStr;
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::{Component, Path, PathBuf};

use log::{error, warn};
use syn::{ItemUse, PathSegment, UseTree};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum UsePathComponent {
  Normal(String),
  ParentDir,
  RootDir,
}

impl UsePathComponent {
  pub fn is_normal(&self) -> bool {
    matches!(self, UsePathComponent::Normal(_))
  }

  pub fn is_parent(&self) -> bool {
    matches!(self, UsePathComponent::ParentDir)
  }

  pub fn is_root(&self) -> bool {
    matches!(self, UsePathComponent::RootDir)
  }
}

impl Default for UsePathComponent {
  fn default() -> Self {
    Self::Normal(String::new())
  }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct PathFs;
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct PathRs;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct UsePath<Type = PathRs> {
  pub path: Vec<UsePathComponent>,
  pub name: String,
  pub actual_name: Option<String>,
  pub local_use: bool,
  pub _type_marker: PhantomData<Type>,
}

impl UsePath<PathRs> {
  pub fn new(path: Vec<UsePathComponent>, name: String, actual_name: Option<String>) -> Self {
    Self {
      path,
      name,
      actual_name,
      local_use: false,
      _type_marker: Default::default(),
    }
  }

  pub fn extended(&self, item: String) -> Self {
    let mut new = (*self).clone();

    new.path.push(match item.as_str() {
      "super" => UsePathComponent::ParentDir,
      "crate" => UsePathComponent::RootDir,
      _ => UsePathComponent::Normal(item),
    });

    new
  }

  pub fn with_name(mut self, name: String) -> Self {
    self.name = name;
    self
  }

  pub fn with_actual_name(mut self, name: String) -> Self {
    self.actual_name = Some(name);
    self
  }

  pub fn with_local_use(mut self, local_use: bool) -> Self {
    self.local_use = local_use;
    self
  }

  pub fn parse_item_use(item_use: &ItemUse) -> Vec<Self> {
    if item_use.leading_colon.is_some() {
      return vec![];
    }

    expand_use_tree_(&UsePath::default(), &item_use.tree)
  }

  pub fn resolve_type_from_uses<'u, 'p>(
    uses: &'u [UsePath<PathRs>],
    path: &'p syn::Path,
  ) -> Result<&'u UsePath<PathRs>, &'p PathSegment> {
    let path_first = path
      .segments
      .first()
      .expect("unexpected empty path")
      .ident
      .to_string();

    let import_item = uses.iter().find(|it| it.name.eq(&path_first));

    if let Some(item) = import_item {
      // todo
      Ok(item)
    } else {
      Err(path.segments.last().expect("unexpected empty path"))
    }
  }

  pub fn is_absolute(&self) -> bool {
    if let Some(first) = self.path.first() {
      *first == UsePathComponent::RootDir
    } else {
      false
    }
  }

  pub fn is_relative(&self) -> bool {
    if let Some(first) = self.path.first() {
      *first == UsePathComponent::ParentDir || first.is_normal()
    } else {
      false
    }
  }

  pub fn is_start_with_parent(&self) -> bool {
    if let Some(first) = self.path.first() {
      *first == UsePathComponent::ParentDir
    } else {
      false
    }
  }

  pub fn to_fs(&self) -> UsePath<PathFs> {
    UsePath::<PathFs> {
      path: self.path.clone(),
      name: self.name.clone(),
      actual_name: self.actual_name.clone(),
      local_use: self.local_use,
      _type_marker: Default::default(),
    }
  }

  pub fn relative_from(&self, location: &UsePath<PathFs>) -> UsePath<PathRs> {
    let mut location = location.path.clone();
    location.pop();

    let import_path = if !self.is_absolute() {
      // region resolve relative path
      let parent_count = self
        .path
        .iter()
        .filter(|it| matches!(it, UsePathComponent::ParentDir))
        .count();
      if parent_count > location.len() {
        error!(
          "detected a escaping import: {:?}, at: {:?}; this will be remain unresolved",
          self.path, location
        );
        return self.clone();
      }

      if parent_count > 0 {
        let mut parent_resolved = location
          .iter()
          .rev()
          .skip(parent_count)
          .rev()
          .cloned()
          .collect::<Vec<_>>();
        parent_resolved.extend(self.path.iter().skip(parent_count).cloned());
        parent_resolved
      } else {
        self.path.clone()
      }
      // endregion
    } else {
      let mut path = self.path.clone();
      path.remove(0);
      path
    };

    let shared_count = location
      .iter()
      .zip(import_path.iter())
      .take_while(|(a, b)| a.eq(b))
      .count();

    let need_go_up = location.len() - shared_count;

    let import_path = itertools::repeat_n(UsePathComponent::ParentDir, need_go_up)
      .chain(import_path.iter().skip(shared_count).cloned())
      .collect::<Vec<_>>();

    let mut result = self.clone();
    result.path = import_path;
    result
  }
}

impl UsePath<PathFs> {
  pub fn new(path: Vec<UsePathComponent>) -> Self {
    Self {
      path,
      name: String::new(),
      actual_name: None,
      local_use: false,
      _type_marker: Default::default(),
    }
  }

  pub fn to_path_buf_with_ext(&self) -> PathBuf {
    let mut path_buf: PathBuf = self.clone().into();
    path_buf.set_extension("rs");
    path_buf
  }

  pub fn to_relative(mut self) -> Self {
    if self.path.first().cloned() == Some(UsePathComponent::RootDir) {
      self.path.remove(0);
      self
    } else {
      self
    }
  }

  pub fn to_rs(&self) -> UsePath<PathRs> {
    UsePath::<PathRs> {
      path: self.path.clone(),
      name: self.name.clone(),
      actual_name: self.actual_name.clone(),
      local_use: self.local_use,
      _type_marker: Default::default(),
    }
  }
}

impl<P: AsRef<Path>> From<P> for UsePath<PathFs> {
  fn from(value: P) -> Self {
    let mut path = value.as_ref().to_path_buf();
    path.set_extension("");

    let path = path
      .components()
      .filter_map(|it| match it {
        Component::RootDir => Some(UsePathComponent::RootDir),
        Component::ParentDir => Some(UsePathComponent::ParentDir),
        Component::Normal(it) => Some(UsePathComponent::Normal(it.to_string_lossy().to_string())),
        Component::Prefix(_) | Component::CurDir => None,
      })
      .collect();

    Self {
      path,
      name: String::new(),
      actual_name: None,
      local_use: false,
      _type_marker: PhantomData::default(),
    }
  }
}

impl From<UsePath<PathFs>> for PathBuf {
  fn from(value: UsePath<PathFs>) -> Self {
    let use_path_iter = value.path.iter().map(|it| match it {
      UsePathComponent::Normal(it) => Component::Normal(OsStr::new(it)),
      UsePathComponent::ParentDir => Component::ParentDir,
      UsePathComponent::RootDir => Component::RootDir,
    });

    let first = value.path.first().unwrap();
    if first.is_normal() {
      const CUR: [Component; 1] = [Component::CurDir];

      PathBuf::from_iter(CUR.into_iter().chain(use_path_iter))
    } else {
      PathBuf::from_iter(use_path_iter)
    }
  }
}

fn expand_use_tree_(prefix: &UsePath<PathRs>, use_tree: &UseTree) -> Vec<UsePath<PathRs>> {
  match use_tree {
    UseTree::Path(use_path) => expand_use_tree_(
      &prefix.extended(use_path.ident.to_string()),
      use_path.tree.deref(),
    ),
    UseTree::Group(use_group) => use_group
      .items
      .iter()
      .flat_map(|it| expand_use_tree_(prefix, it))
      .collect(),
    UseTree::Name(use_name) => {
      vec![prefix.clone().with_name(use_name.ident.to_string())]
    }
    UseTree::Rename(use_rename) => {
      vec![prefix
        .clone()
        .with_name(use_rename.rename.to_string())
        .with_actual_name(use_rename.ident.to_string())]
    }
    UseTree::Glob(_) => {
      warn!("detected a use statement with *, this is unsupported, any type imported by this will be ignored");
      vec![]
    }
  }
}
