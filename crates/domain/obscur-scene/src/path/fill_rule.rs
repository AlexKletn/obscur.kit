//! How a path's interior is determined.

/// How a path's interior is determined from its (possibly self-intersecting,
/// multi-subpath) outline (spec §5.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FillRule {
    /// Non-zero winding rule: a point is inside when the signed number of times
    /// the outline winds around it is non-zero. This is the default and matches
    /// most vector authoring tools.
    NonZero,
    /// Even-odd rule: a point is inside when a ray from it crosses the outline
    /// an odd number of times. Overlapping subpaths cancel out.
    EvenOdd,
}

impl Default for FillRule {
    /// The default fill rule is [`FillRule::NonZero`].
    fn default() -> Self {
        Self::NonZero
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_non_zero() {
        assert_eq!(FillRule::default(), FillRule::NonZero);
    }
}
