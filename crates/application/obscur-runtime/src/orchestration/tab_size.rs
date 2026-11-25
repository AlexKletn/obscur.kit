/// The resolved `tab-size` value (CSS Text 3 §4.3): a `<number>` multiple of the
/// space advance, or a fixed `<length>` tab-stop width in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum TabSize {
    /// `tab-size: <number>` — the tab stop is `factor` × the advance of a space.
    Factor(f32),
    /// `tab-size: <length>` — the tab stop is this fixed pixel width.
    Length(f32),
}

impl TabSize {
    /// Resolves the tab-stop width in logical pixels given the `space_advance` of
    /// the node's primary font.
    #[must_use]
    pub(crate) fn resolve(self, space_advance: f32) -> f32 {
        match self {
            Self::Factor(factor) => factor * space_advance,
            Self::Length(px) => px,
        }
    }
}
