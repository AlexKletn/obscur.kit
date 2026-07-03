//! Renderer port (skeleton).

/// A replaceable renderer backend. Backends consume immutable Scene/Paint
/// snapshots; only rasterization may differ between them (spec §5.2). This
/// skeleton exposes identification only.
pub trait Renderer {
    /// A stable, human-readable backend name for diagnostics.
    fn backend_name(&self) -> &str;
}
