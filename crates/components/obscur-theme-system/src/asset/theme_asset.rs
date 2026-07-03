//! A font/icon/image/stylesheet resource bundled by a theme (spec §2.6).

use std::sync::Arc;

/// The kind of resource a [`ThemeAsset`] carries (spec §2.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    /// A font file (e.g. TTF/OTF/WOFF).
    Font,
    /// An icon resource.
    Icon,
    /// A raster or vector image.
    Image,
    /// A stylesheet.
    Stylesheet,
}

/// Where a [`ThemeAsset`]'s bytes come from (spec §2.6).
///
/// An asset is either baked into the theme as bytes, or referenced by a path the
/// host resolves at load time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetSource {
    /// Bytes embedded directly in the theme, shared via [`Arc`].
    Embedded(Arc<[u8]>),
    /// A path the host resolves to load the asset.
    Path(Box<str>),
}

/// A named font/icon/image/stylesheet resource a theme bundles (spec §2.6).
///
/// Each asset pairs a name and an [`AssetKind`] with an [`AssetSource`] that is
/// either embedded bytes or a host-resolved path. Because the payloads
/// ([`Arc<[u8]>`](Arc) and [`Box<str>`]) are `Eq`, this type derives `Debug`,
/// `Clone`, `PartialEq`, and `Eq`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeAsset {
    /// The asset's name (e.g. `"font.body"` or `"icon.close"`).
    pub name: Box<str>,
    /// The kind of resource.
    pub kind: AssetKind,
    /// Where the asset's data comes from.
    pub source: AssetSource,
}

impl ThemeAsset {
    /// Creates an asset whose bytes are embedded directly in the theme.
    #[must_use]
    pub fn embedded(name: impl Into<Box<str>>, kind: AssetKind, bytes: impl Into<Arc<[u8]>>) -> Self {
        Self {
            name: name.into(),
            kind,
            source: AssetSource::Embedded(bytes.into()),
        }
    }

    /// Creates an asset referenced by a host-resolved path.
    #[must_use]
    pub fn path(name: impl Into<Box<str>>, kind: AssetKind, path: impl Into<Box<str>>) -> Self {
        Self {
            name: name.into(),
            kind,
            source: AssetSource::Path(path.into()),
        }
    }

    /// Returns the asset's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the asset's kind.
    #[must_use]
    pub const fn kind(&self) -> AssetKind {
        self.kind
    }

    /// Returns the asset's source.
    #[must_use]
    pub const fn source(&self) -> &AssetSource {
        &self.source
    }

    /// Returns the embedded bytes, or `None` for a path-backed asset.
    #[must_use]
    pub fn bytes(&self) -> Option<&[u8]> {
        match &self.source {
            AssetSource::Embedded(bytes) => Some(bytes),
            AssetSource::Path(_) => None,
        }
    }

    /// Returns whether the asset's bytes are embedded (rather than a path).
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
        assert_eq!(asset.source(), &AssetSource::Embedded(Arc::from(vec![1u8, 2, 3])));
    }

    #[test]
    fn path_has_no_bytes() {
        let asset = ThemeAsset::path("icon.close", AssetKind::Icon, "icons/close.svg");
        assert_eq!(asset.kind(), AssetKind::Icon);
        assert!(!asset.is_embedded());
        assert_eq!(asset.bytes(), None);
        assert_eq!(asset.source(), &AssetSource::Path(Box::from("icons/close.svg")));
    }

    #[test]
    fn embedded_from_slice() {
        let asset = ThemeAsset::embedded("style.main", AssetKind::Stylesheet, Arc::<[u8]>::from(&b"body{}"[..]));
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
