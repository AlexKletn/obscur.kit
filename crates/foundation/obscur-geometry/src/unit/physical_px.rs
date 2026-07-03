//! Physical (device) pixel scalar.

/// A length in physical device pixels, produced by the renderer from logical
/// units and a scale factor (spec §4.5).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct PhysicalPx(pub f32);
