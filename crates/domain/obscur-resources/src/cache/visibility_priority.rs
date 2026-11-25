#[doc = include_str!("../../docs/api/cache/visibility_priority.doc/VisibilityPriority.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VisibilityPriority {
    /// Not shown at all; the first artifact to evict.
    Hidden,
    /// Fully off-screen but still live.
    Offscreen,
    /// Just outside the viewport; likely to be needed soon.
    NearViewport,
    /// Currently on screen; retained longest.
    Visible,
}

#[cfg(test)]
mod tests {
    use super::VisibilityPriority;

    #[test]
    fn visibility_orders_hidden_smallest() {
        assert!(VisibilityPriority::Hidden < VisibilityPriority::Offscreen);
        assert!(VisibilityPriority::Offscreen < VisibilityPriority::NearViewport);
        assert!(VisibilityPriority::NearViewport < VisibilityPriority::Visible);
    }
}
