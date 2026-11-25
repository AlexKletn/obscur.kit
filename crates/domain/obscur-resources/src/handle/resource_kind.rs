#[doc = include_str!("../../docs/api/handle/resource_kind.doc/ResourceKind.md")]
//
// Note: only `Image`, `Font`, `Stylesheet`, and `Shader` have a dedicated
// typed-handle newtype in this module's layout. `Svg`, `Atlas`, and
// `EmbeddedAsset` are represented only via the generic [`ResourceHandle`] plus
// their `ResourceKind`; that omission is intentional.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceKind {
    /// A raster or decoded image resource.
    Image,
    /// A font resource.
    Font,
    /// A CSS stylesheet resource.
    Stylesheet,
    /// A shader program resource.
    Shader,
    /// An SVG (vector graphics) resource.
    Svg,
    /// A texture atlas resource.
    Atlas,
    /// An embedded asset baked into the binary.
    EmbeddedAsset,
}
