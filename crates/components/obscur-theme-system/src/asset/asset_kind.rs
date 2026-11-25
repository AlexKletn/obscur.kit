/// The kind of resource a [`ThemeAsset`](super::ThemeAsset) carries (spec §2.6).
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
