/// A color stop's position while lowering a `repeating-linear-gradient`, before
/// normalization: either an absolute `px` offset or a `<percentage>` fraction of
/// the gradient line. The two variants are the two period units; a single
/// repeating gradient uses one of them consistently.
#[derive(Clone, Copy)]
pub(super) enum RepeatPos {
    /// An absolute px offset (the common stripe case).
    Px(f32),
    /// A percentage offset, as an authored `0..=100` value.
    Pct(f32),
}

impl RepeatPos {
    /// The raw magnitude in this stop's own unit (px or percent value).
    pub(super) fn value(self) -> f32 {
        match self {
            Self::Px(v) | Self::Pct(v) => v,
        }
    }

    /// Whether this position is expressed in `px` (vs. a percentage).
    pub(super) fn is_px(self) -> bool {
        matches!(self, Self::Px(_))
    }
}
