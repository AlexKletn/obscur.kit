use super::intrinsic_track_kind::Kind;

/// A track's resolved *min* sizing function (§11.4). A `Percent`/`Fixed` min is
/// a definite length; `auto`, `min-content`, `max-content` (and a standalone /
/// `minmax` `fr` min) are intrinsic.
#[derive(Clone, Copy)]
pub(super) enum MinFn {
    /// A definite length (fixed px or a resolved percentage).
    Fixed(f32),
    /// An intrinsic minimum — grows to fit the [`Kind`]'s content contribution.
    Intrinsic(Kind),
}

impl MinFn {
    /// The contribution kind this min sizing function is fed by, or [`None`] when
    /// it is a definite length that takes no contribution at all.
    pub(super) fn kind(self) -> Option<Kind> {
        match self {
            Self::Fixed(_) => None,
            Self::Intrinsic(kind) => Some(kind),
        }
    }
}
