#[doc = include_str!("../../../docs/api/effects/gaussian_blur.doc/CompositeMask.md")]
#[derive(Clone, Copy, Default)]
pub(super) struct CompositeMask {
    /// Panel border box `[x0, y0, x1, y1]` in the quad's pixel space.
    pub(super) panel: [f32; 4],
    /// Per-corner radii `(tl, tr, br, bl)` in quad pixels.
    pub(super) radii: [f32; 4],
    /// The composited quad's `(width, height)` in pixels; `[0, 0]` disables the mask.
    pub(super) quad: [f32; 2],
    /// Antialiased edge half-width, in pixels.
    pub(super) aa: f32,
}

impl CompositeMask {
    /// The shader's `mask_enabled` flag: `1` when a non-degenerate quad is set,
    /// else `0` (the disabled default → whole-quad composite).
    pub(super) fn enabled(&self) -> f32 {
        if self.quad[0] > 0.0 && self.quad[1] > 0.0 {
            1.0
        } else {
            0.0
        }
    }
}
