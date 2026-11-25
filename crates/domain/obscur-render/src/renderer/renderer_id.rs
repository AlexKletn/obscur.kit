//! A stable identity for a renderer backend instance (spec §5.2).

#[doc = include_str!("../../docs/api/renderer/renderer_id.doc/RendererId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RendererId(u64);

impl RendererId {
    /// Wraps a raw renderer identity value.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw renderer identity value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_raw_round_trips() {
        assert_eq!(RendererId::from_raw(7).raw(), 7);
    }

    #[test]
    fn distinct_ids_compare_and_order() {
        let a = RendererId::from_raw(1);
        let b = RendererId::from_raw(2);
        assert_ne!(a, b);
        assert!(a < b);
        assert_eq!(a, RendererId::from_raw(1));
    }
}
