//! Stable, non-CSS identifiers a test or DevTools client targets a node by
//! (spec §7.5).

use obscur_identity::InternedSymbol;

#[doc = include_str!("../../docs/api/view/stable_selector.doc/TEST_ID.md")]
pub const TEST_ID: InternedSymbol = InternedSymbol::from_raw(0x0000_0201);

#[doc = include_str!("../../docs/api/view/stable_selector.doc/AUTOMATION_ID.md")]
pub const AUTOMATION_ID: InternedSymbol = InternedSymbol::from_raw(0x0000_0202);

#[cfg(test)]
mod tests {
    use super::{AUTOMATION_ID, TEST_ID};
    use crate::view::{CHECKED, CLASS, DISABLED, EVENT_CLICK, ID, LOCAL_NAME, READONLY, SRC, TYPE};

    #[test]
    fn the_stable_selectors_collide_with_no_other_reserved_symbol() {
        for other in [
            CLASS,
            ID,
            TYPE,
            LOCAL_NAME,
            SRC,
            DISABLED,
            CHECKED,
            READONLY,
            EVENT_CLICK,
        ] {
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
