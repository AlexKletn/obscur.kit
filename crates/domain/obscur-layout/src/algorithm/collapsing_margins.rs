#[doc = include_str!("../../docs/api/algorithm/flow_layout.doc/CollapsingMargins.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct CollapsingMargins {
    /// The largest non-negative margin in the set (`0` if none).
    max_positive: f32,
    /// The most-negative margin in the set (`0` if none).
    min_negative: f32,
}

impl CollapsingMargins {
    /// The empty set — no margins have collapsed in yet (used margin `0`).
    pub(super) const EMPTY: Self = Self {
        max_positive: 0.0,
        min_negative: 0.0,
    };

    /// Folds one margin into the set, keeping the running positive/negative
    /// extremes. Order-independent, so an entire adjoining run reduces to the
    /// same result however it is accumulated.
    #[must_use]
    pub(super) fn adjoin(self, margin: f32) -> Self {
        if margin >= 0.0 {
            Self {
                max_positive: self.max_positive.max(margin),
                ..self
            }
        } else {
            Self {
                min_negative: self.min_negative.min(margin),
                ..self
            }
        }
    }

    /// The used margin for the collapsed set: `max_positive + min_negative`.
    #[must_use]
    pub(super) fn solve(self) -> f32 {
        self.max_positive + self.min_negative
    }
}
