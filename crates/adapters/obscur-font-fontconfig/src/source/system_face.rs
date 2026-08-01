#![doc = include_str!("system_face.doc/_module.md")]

use std::path::{Path, PathBuf};

use obscur_text::font::FontAttributes;

#[doc = include_str!("system_face.doc/SystemFace.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemFace {
    family: Box<str>,
    path: PathBuf,
    attributes: FontAttributes,
}

impl SystemFace {
    #[doc = include_str!("system_face.doc/new.md")]
    #[must_use]
    pub fn new(
        family: impl Into<Box<str>>,
        path: impl Into<PathBuf>,
        attributes: FontAttributes,
    ) -> Self {
        Self {
            family: family.into(),
            path: path.into(),
            attributes,
        }
    }

    #[doc = include_str!("system_face.doc/family.md")]
    #[must_use]
    pub fn family(&self) -> &str {
        &self.family
    }

    #[doc = include_str!("system_face.doc/path.md")]
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[doc = include_str!("system_face.doc/attributes.md")]
    #[must_use]
    pub const fn attributes(&self) -> &FontAttributes {
        &self.attributes
    }
}
