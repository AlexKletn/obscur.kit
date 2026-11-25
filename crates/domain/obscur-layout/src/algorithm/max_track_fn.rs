use super::intrinsic_track_kind::Kind;

/// A track's resolved *max* sizing function (§11.4).
#[derive(Clone, Copy)]
pub(super) enum MaxFn {
    /// A definite length (fixed px or a resolved percentage): the growth limit.
    Fixed(f32),
    /// An intrinsic maximum — an infinite initial growth limit, grown to fit the
    /// [`Kind`]'s content contribution.
    Intrinsic(Kind),
    /// A flexible (`fr`) maximum with the given flex factor.
    Flex(f32),
}

impl MaxFn {
    /// The contribution kind this max sizing function is fed by; [`None`] for a
    /// definite length and for `fr` (whose growth limit is settled by §11.7, not
    /// by content).
    pub(super) fn kind(self) -> Option<Kind> {
        match self {
            Self::Fixed(_) | Self::Flex(_) => None,
            Self::Intrinsic(kind) => Some(kind),
        }
    }
}
