#![doc = include_str!("cascade_origin.doc/_module.md")]

#[doc = include_str!("cascade_origin.doc/CascadeOrigin.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CascadeOrigin {
    #[doc = include_str!("cascade_origin.doc/SystemDefault.md")]
    SystemDefault,
    #[doc = include_str!("cascade_origin.doc/Theme.md")]
    Theme,
    #[doc = include_str!("cascade_origin.doc/GlobalApp.md")]
    GlobalApp,
    #[doc = include_str!("cascade_origin.doc/ComponentModule.md")]
    ComponentModule,
    #[doc = include_str!("cascade_origin.doc/Inline.md")]
    Inline,
    #[doc = include_str!("cascade_origin.doc/RuntimeOverride.md")]
    RuntimeOverride,
}

impl CascadeOrigin {
    #[doc = include_str!("cascade_origin.doc/is_runtime_override.md")]
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
