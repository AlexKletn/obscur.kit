//! Screen-size thresholds for scene object detail.

/// Defines zero-based levels of detail selected by physical pixel extent.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DetailLevels {
    thresholds: Vec<f32>,
}

impl DetailLevels {
    /// Creates a policy from strictly increasing positive thresholds. Each threshold starts the next level.
    #[must_use]
    pub fn new(thresholds: Vec<f32>) -> Option<Self> {
        if thresholds
            .iter()
            .any(|threshold| !threshold.is_finite() || *threshold <= 0.0)
            || thresholds.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return None;
        }
        Some(Self { thresholds })
    }

    /// Returns the detail level at the supplied physical pixel extent. Exact thresholds enter the higher level.
    #[must_use]
    pub fn level_for(&self, physical_extent: f32) -> usize {
        self.thresholds
            .partition_point(|&threshold| threshold <= physical_extent)
    }
}
