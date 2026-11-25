/// The result of placing one flex line.
pub(super) struct PlacedLine {
    /// The line's cross extent, used to stack lines (the stretch target for the
    /// single-line path, the max item cross for the multi-line path).
    pub(super) cross: f32,
    /// The line's **pre-stretch** content cross extent — the tallest item's
    /// hypothetical (measured or explicit) cross size. Equals [`Self::cross`] for
    /// the multi-line path; for the single-line path it is the height an
    /// (auto, indefinite) row hugs to. Only meaningful when the caller set
    /// `content_cross_needed`.
    pub(super) content_cross: f32,
}
