/// The comparison a [`FeatureQuery`](super::FeatureQuery) applies between the environment value and
/// its reference length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionComparator {
    /// Strictly less than (`<`).
    Lt,
    /// Less than or equal (`<=`, and the `max-*` prefix forms).
    Le,
    /// Equal (`=`, and a plain `(width: 600px)` colon form).
    Eq,
    /// Greater than or equal (`>=`, and the `min-*` prefix forms).
    Ge,
    /// Strictly greater than (`>`).
    Gt,
}

impl ConditionComparator {
    pub(super) fn compare(self, actual: f32, reference: f32) -> bool {
        match self {
            Self::Lt => actual < reference,
            Self::Le => actual <= reference,
            Self::Eq => (actual - reference).abs() < f32::EPSILON,
            Self::Ge => actual >= reference,
            Self::Gt => actual > reference,
        }
    }
}
