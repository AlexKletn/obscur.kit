//! Stable, non-CSS identifiers a test or DevTools client targets a node by
//! (spec §7.5).

use obscur_identity::InternedSymbol;

/// Reserved symbol for the `test-id` attribute — a node's stable testing
/// identifier (spec §7.5).
///
/// # Deliberately not a style attribute
///
/// Spec §7.5 requires that these identifiers *do not affect layout, do not
/// affect CSS specificity, and do not affect visual output*. That is enforced
/// here by construction rather than by convention: `test-id` is **not** part of
/// [`StyleAttributes`](super::StyleAttributes), so
/// [`extract`](super::StyleAttributes::extract) never sees it and the selector
/// matcher has nothing to match on. It lives only in the node's retained
/// property list, which is where a resolver reads it.
///
/// It occupies a namespace of its own (`0x0000_02xx`), distinct from both the
/// CSS-relevant reserved attributes (`0x0000_000x`) and the reserved events
/// (`0x0000_010x`), so the separation is visible in the handle itself.
///
/// Being an ordinary property also means it satisfies §7.5's "may be compiled
/// out of production builds": a build that does not author it simply carries no
/// such property, and costs nothing.
pub const TEST_ID: InternedSymbol = InternedSymbol::from_raw(0x0000_0201);

/// Reserved symbol for the `automation-id` attribute — a node's stable identifier
/// for external automation and assistive tooling (spec §7.5).
///
/// # Why it is not `test-id`
///
/// §7.5 lists the two separately, and they answer to different owners. A `test-id`
/// belongs to the app's own test suite: it is churn-friendly, renamed whenever a
/// test is rewritten, and may be compiled out. An `automation-id` is a contract
/// with software outside the codebase — an automation harness or an accessibility
/// tool — which cannot be updated in the same commit that renames it.
///
/// Collapsing them into one attribute would force those two lifetimes onto one
/// string: either tests stop renaming freely, or external automation breaks when
/// they do. They share every other property (not a style attribute, no effect on
/// matching, layout or visuals) and the same `0x0000_02xx` namespace.
pub const AUTOMATION_ID: InternedSymbol = InternedSymbol::from_raw(0x0000_0202);

#[cfg(test)]
mod tests {
    use super::{AUTOMATION_ID, TEST_ID};
    use crate::view::{CHECKED, CLASS, DISABLED, EVENT_CLICK, ID, READONLY, SRC, TYPE};

    #[test]
    fn the_stable_selectors_collide_with_no_other_reserved_symbol() {
        for other in [CLASS, ID, TYPE, SRC, DISABLED, CHECKED, READONLY, EVENT_CLICK] {
            assert_ne!(TEST_ID, other);
            assert_ne!(AUTOMATION_ID, other);
        }
        assert_ne!(TEST_ID, AUTOMATION_ID, "two identifiers, two symbols");
    }

    #[test]
    fn neither_stable_selector_is_a_style_attribute() {
        // The §7.5 guarantee, checked rather than asserted in prose: a node
        // carrying only a `test-id` extracts no style attributes at all, so it
        // cannot influence matching, specificity or layout.
        use crate::view::{StyleAttributes, ViewProperty, ViewValue};

        let attributes = StyleAttributes::extract(&[
            ViewProperty::new(TEST_ID, ViewValue::Text("submit".into())),
            ViewProperty::new(AUTOMATION_ID, ViewValue::Text("submit".into())),
        ]);
        assert_eq!(attributes, StyleAttributes::new());
    }
}
