//! One face fontconfig reported: where it lives and what cut it is.

use std::path::{Path, PathBuf};

use obscur_text::font::FontAttributes;

#[doc = include_str!("../../docs/api/source/system_face.doc/SystemFace.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemFace {
    family: Box<str>,
    path: PathBuf,
    attributes: FontAttributes,
}

impl SystemFace {
    /// Creates a located face.
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

    /// The family name fontconfig reported — the face it *actually* resolved,
    /// which for a generic request is the host's nominated substitute rather than
    /// the generic's own name.
    #[must_use]
    pub fn family(&self) -> &str {
        &self.family
    }

    /// The font file's path on the host.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The face's weight/slant/width, converted from fontconfig's scales to CSS's.
    #[must_use]
    pub const fn attributes(&self) -> &FontAttributes {
        &self.attributes
    }
}
