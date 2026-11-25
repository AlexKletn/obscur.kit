//! Whether pointer input stops at a layer surface or passes through it.

#[doc = include_str!("../../docs/api/layer/layer_pointer_policy.doc/LayerPointerPolicy.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LayerPointerPolicy {
    #[doc = include_str!("../../docs/api/layer/layer_pointer_policy.doc/Interactive.md")]
    #[default]
    Interactive,
    #[doc = include_str!("../../docs/api/layer/layer_pointer_policy.doc/Transparent.md")]
    Transparent,
}

impl LayerPointerPolicy {
    /// Whether this policy routes any pointer event to the surface at all.
    #[must_use]
    pub const fn takes_pointer(self) -> bool {
        matches!(self, Self::Interactive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_takes_the_pointer() {
        // The opposite default would make every panel click-through, which is
        // the surprising half of the pair: a surface nobody can click is a
        // choice, not a starting point.
        assert_eq!(
            LayerPointerPolicy::default(),
            LayerPointerPolicy::Interactive
        );
        assert!(LayerPointerPolicy::default().takes_pointer());
    }

    #[test]
    fn a_transparent_surface_routes_nothing_to_itself() {
        assert!(!LayerPointerPolicy::Transparent.takes_pointer());
    }
}
