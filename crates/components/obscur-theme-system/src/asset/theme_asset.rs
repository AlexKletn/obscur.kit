#![doc = include_str!("theme_asset.doc/_module.md")]

use std::sync::Arc;

#[doc = include_str!("theme_asset.doc/AssetKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    #[doc = include_str!("theme_asset.doc/Font.md")]
    Font,
    #[doc = include_str!("theme_asset.doc/Icon.md")]
    Icon,
    #[doc = include_str!("theme_asset.doc/Image.md")]
    Image,
    #[doc = include_str!("theme_asset.doc/Stylesheet.md")]
    Stylesheet,
}

#[doc = include_str!("theme_asset.doc/AssetSource.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetSource {
    #[doc = include_str!("theme_asset.doc/Embedded.md")]
    Embedded(Arc<[u8]>),
    #[doc = include_str!("theme_asset.doc/Path.md")]
    Path(Box<str>),
}

#[doc = include_str!("theme_asset.doc/ThemeAsset.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeAsset {
    #[doc = include_str!("theme_asset.doc/name.md")]
    pub name: Box<str>,
    #[doc = include_str!("theme_asset.doc/kind.md")]
    pub kind: AssetKind,
    #[doc = include_str!("theme_asset.doc/source.md")]
    pub source: AssetSource,
}

impl ThemeAsset {
    #[doc = include_str!("theme_asset.doc/embedded.md")]
    #[must_use]
    pub fn embedded(
        name: impl Into<Box<str>>,
        kind: AssetKind,
        bytes: impl Into<Arc<[u8]>>,
    ) -> Self {
        Self {
            name: name.into(),
            kind,
            source: AssetSource::Embedded(bytes.into()),
        }
    }

    #[doc = include_str!("theme_asset.doc/path.md")]
    #[must_use]
    pub fn path(name: impl Into<Box<str>>, kind: AssetKind, path: impl Into<Box<str>>) -> Self {
        Self {
            name: name.into(),
            kind,
            source: AssetSource::Path(path.into()),
        }
    }

    #[doc = include_str!("theme_asset.doc/name_2.md")]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[doc = include_str!("theme_asset.doc/kind_2.md")]
    #[must_use]
    pub const fn kind(&self) -> AssetKind {
        self.kind
    }

    #[doc = include_str!("theme_asset.doc/source_2.md")]
    #[must_use]
    pub const fn source(&self) -> &AssetSource {
        &self.source
    }

    #[doc = include_str!("theme_asset.doc/bytes.md")]
    #[must_use]
    pub fn bytes(&self) -> Option<&[u8]> {
        match &self.source {
            AssetSource::Embedded(bytes) => Some(bytes),
            AssetSource::Path(_) => None,
        }
    }

    #[doc = include_str!("theme_asset.doc/is_embedded.md")]
    #[must_use]
    pub const fn is_embedded(&self) -> bool {
        matches!(self.source, AssetSource::Embedded(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_exposes_bytes() {
        let asset = ThemeAsset::embedded("font.body", AssetKind::Font, vec![1u8, 2, 3]);
        assert_eq!(asset.name(), "font.body");
        assert_eq!(asset.kind(), AssetKind::Font);
        assert!(asset.is_embedded());
        assert_eq!(asset.bytes(), Some(&[1u8, 2, 3][..]));
        assert_eq!(
            asset.source(),
            &AssetSource::Embedded(Arc::from(vec![1u8, 2, 3]))
        );
    }

    #[test]
    fn path_has_no_bytes() {
        let asset = ThemeAsset::path("icon.close", AssetKind::Icon, "icons/close.svg");
        assert_eq!(asset.kind(), AssetKind::Icon);
        assert!(!asset.is_embedded());
        assert_eq!(asset.bytes(), None);
        assert_eq!(
            asset.source(),
            &AssetSource::Path(Box::from("icons/close.svg"))
        );
    }

    #[test]
    fn embedded_from_slice() {
        let asset = ThemeAsset::embedded(
            "style.main",
            AssetKind::Stylesheet,
            Arc::<[u8]>::from(&b"body{}"[..]),
        );
        assert!(asset.is_embedded());
        assert_eq!(asset.bytes(), Some(&b"body{}"[..]));
    }

    #[test]
    fn image_kind() {
        let asset = ThemeAsset::path("image.logo", AssetKind::Image, "logo.png");
        assert_eq!(asset.kind(), AssetKind::Image);
        assert!(!asset.is_embedded());
    }
}
