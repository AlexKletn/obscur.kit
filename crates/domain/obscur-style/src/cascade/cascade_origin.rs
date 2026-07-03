//! The origin a declaration comes from: [`CascadeOrigin`] (§4.14).

/// The origin of a declaration in the cascade, ordered by ascending precedence
/// for *normal* (non-`!important`) declarations (§4.14).
///
/// The six origins, from lowest to highest precedence for normal declarations:
///
/// 1. [`CascadeOrigin::SystemDefault`] — the built-in system default styles;
/// 2. [`CascadeOrigin::Theme`] — the active theme stylesheet;
/// 3. [`CascadeOrigin::GlobalApp`] — global, application-wide CSS;
/// 4. [`CascadeOrigin::ComponentModule`] — component / module-scoped CSS;
/// 5. [`CascadeOrigin::Inline`] — inline CSS set directly on a node;
/// 6. [`CascadeOrigin::RuntimeOverride`] — an explicit runtime override layer.
///
/// The variants are declared in this ascending order and the type derives
/// [`Ord`], so the derived ordering *is* the normal-declaration precedence:
/// `SystemDefault < Theme < GlobalApp < ComponentModule < Inline <
/// RuntimeOverride`.
///
/// Note that this ordering only directly governs *normal* declarations; how
/// `!important` and the runtime-override layer interleave with authored CSS is
/// handled by [`crate::cascade::CascadePriority`], which builds its band from
/// both the origin and the declaration's `!important` flag.
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
    /// Returns `true` when this origin is the runtime-override layer.
    ///
    /// The runtime-override layer is special: it sits above all authored
    /// declarations, normal or `!important` (§4.14).
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
