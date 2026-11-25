//! Stroke corner-join style.

/// The shape drawn where two segments of a stroked path meet (spec §5.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineJoin {
    /// Extend the outer edges until they meet at a sharp point, subject to the
    /// stroke's miter limit (beyond which the join falls back to a bevel).
    Miter,
    /// Round off the corner with an arc of radius half the stroke width.
    Round,
    /// Cut the corner off with a straight edge between the two outer endpoints.
    Bevel,
}

impl Default for LineJoin {
    /// The default line join is [`LineJoin::Miter`].
    fn default() -> Self {
        Self::Miter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_miter() {
        assert_eq!(LineJoin::default(), LineJoin::Miter);
    }
}
