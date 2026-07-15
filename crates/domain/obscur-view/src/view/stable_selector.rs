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

#[cfg(test)]
mod tests {
    use super::TEST_ID;
    use crate::view::{CHECKED, CLASS, DISABLED, EVENT_CLICK, ID, READONLY, SRC, TYPE};

    #[test]
    fn test_id_collides_with_no_other_reserved_symbol() {
        for other in [CLASS, ID, TYPE, SRC, DISABLED, CHECKED, READONLY, EVENT_CLICK] {
            assert_ne!(TEST_ID, other);
        }
    }

    #[test]
    fn test_id_is_not_a_style_attribute() {
        // The §7.5 guarantee, checked rather than asserted in prose: a node
        // carrying only a `test-id` extracts no style attributes at all, so it
        // cannot influence matching, specificity or layout.
        use crate::view::{StyleAttributes, ViewProperty, ViewValue};

        let attributes = StyleAttributes::extract(&[ViewProperty::new(
            TEST_ID,
            ViewValue::Text("submit".into()),
        )]);
        assert_eq!(attributes, StyleAttributes::new());
    }
}
