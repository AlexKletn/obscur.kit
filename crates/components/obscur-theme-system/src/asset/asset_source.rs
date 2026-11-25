use std::sync::Arc;

#[doc = include_str!("../../docs/api/asset/theme_asset.doc/AssetSource.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetSource {
    /// Bytes embedded directly in the theme, shared via [`Arc`].
    Embedded(Arc<[u8]>),
    /// A path the host resolves to load the asset.
    Path(Box<str>),
}
