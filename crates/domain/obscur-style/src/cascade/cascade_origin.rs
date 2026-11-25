//! The origin a declaration comes from: [`CascadeOrigin`] (§4.14).

#[doc = include_str!("../../docs/api/cascade/cascade_origin.doc/CascadeOrigin.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CascadeOrigin {
    /// Built-in system default styles — the lowest-precedence origin (§4.14).
    SystemDefault,
    /// The active theme stylesheet (§4.14).
    Theme,
    /// Global, application-wide CSS (§4.14).
    GlobalApp,
    /// Component- or module-scoped CSS (§4.14).
    ComponentModule,
    /// Inline CSS applied directly to a node (§4.14).
    Inline,
    /// The explicit runtime override layer. This layer sits above all authored
    /// CSS — including authored `!important` — so a runtime override always wins
    /// over authored declarations regardless of their importance (§4.14).
    RuntimeOverride,
}

impl CascadeOrigin {
    #[doc = include_str!("../../docs/api/cascade/cascade_origin.doc/is_runtime_override.md")]
    #[must_use]
    pub const fn is_runtime_override(self) -> bool {
        matches!(self, Self::RuntimeOverride)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_ascend_in_precedence_order() {
        assert!(CascadeOrigin::SystemDefault < CascadeOrigin::Theme);
        assert!(CascadeOrigin::Theme < CascadeOrigin::GlobalApp);
        assert!(CascadeOrigin::GlobalApp < CascadeOrigin::ComponentModule);
        assert!(CascadeOrigin::ComponentModule < CascadeOrigin::Inline);
        assert!(CascadeOrigin::Inline < CascadeOrigin::RuntimeOverride);
    }

    #[test]
    fn runtime_override_is_the_maximum() {
        for origin in [
            CascadeOrigin::SystemDefault,
            CascadeOrigin::Theme,
            CascadeOrigin::GlobalApp,
            CascadeOrigin::ComponentModule,
            CascadeOrigin::Inline,
        ] {
            assert!(origin < CascadeOrigin::RuntimeOverride);
        }
    }

    #[test]
    fn is_runtime_override_predicate() {
        assert!(CascadeOrigin::RuntimeOverride.is_runtime_override());
        assert!(!CascadeOrigin::Inline.is_runtime_override());
        assert!(!CascadeOrigin::SystemDefault.is_runtime_override());
    }
}
