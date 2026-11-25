use super::intrinsic_track_kind::Kind;

#[doc = include_str!("../../docs/api/algorithm/grid_track_sizing.doc/Contributions.md")]
#[derive(Clone, Copy, Default)]
pub(super) struct Contributions {
    pub(super) auto: f32,
    pub(super) min: f32,
    pub(super) max: f32,
}

impl Contributions {
    /// This item's contribution for `kind`.
    pub(super) fn of(self, kind: Kind) -> f32 {
        match kind {
            Kind::Auto => self.auto,
            Kind::Min => self.min,
            Kind::Max => self.max,
        }
    }

    /// The contribution slot for `kind`, to fill in from a measure.
    pub(super) fn slot(&mut self, kind: Kind) -> &mut f32 {
        match kind {
            Kind::Auto => &mut self.auto,
            Kind::Min => &mut self.min,
            Kind::Max => &mut self.max,
        }
    }
}
