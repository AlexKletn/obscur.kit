#![doc = include_str!("../../docs/api/validation/outlet_rule.doc/_module.md")]

use obscur_ui_ir::name_binding::OUTLET_TAG;

pub use super::{OutletName, OutletUse};

#[doc = include_str!("../../docs/api/validation/outlet_rule.doc/is_outlet_tag.md")]
#[must_use]
pub fn is_outlet_tag(tag: &str) -> bool {
    tag == OUTLET_TAG
}

/// The diagnostic for a `<slot/>` with no `name` (spec §2.12).
pub const MISSING_NAME_MESSAGE: &str = "`<slot>` requires a `name`: §2.12 declares *named* slots, and an authored outlet is matched to its component's declaration by name. The runtime does have a default slot — the outlet naming none, which takes the content that names none — but nothing spells it in `.okui`: a contract declares it under a reserved name no author can write, and until that is decided an unnamed outlet here would be a declaration nothing validates. Write `<slot name=\"…\"/>`";

/// The diagnostic for a `<slot name={expr}/>` (spec §2.12).
pub const DYNAMIC_NAME_MESSAGE: &str = "`<slot name={…}>` is refused: a slot is matched against its component's declaration at compile time, so an outlet's name must be a literal — the same rule that refuses `slot={…}` on the child side of the same join. Write `<slot name=\"…\"/>`";

/// The diagnostic for children under a `<slot>` (spec §2.12).
pub const OUTLET_CHILDREN_MESSAGE: &str = "`<slot>` takes no children *here*: projection does render an unfilled outlet's children as fallback content, but §2.12 describes no such construct and no authored `.okui` component declares one — the one component that uses it, the built-in `button`, builds its outlet in Rust. Until the spelling is decided, children on an authored outlet would be content nothing validates";

/// The diagnostic for any attribute other than `name` on a `<slot>` (spec §2.12).
#[must_use]
pub fn extra_attribute_message(attribute: &str) -> String {
    format!(
        "unknown attribute '{attribute}' on `<slot>`: an outlet is a position, not an element — projection deletes the node before style, layout or paint can read it, so it carries nothing but its `name`"
    )
}

#[doc = include_str!("../../docs/api/validation/outlet_rule.doc/check_outlet.md")]
#[must_use]
pub fn check_outlet(tag: &str, name: OutletName<'_>, outlet: &OutletUse<'_>) -> Vec<String> {
    if !is_outlet_tag(tag) {
        return Vec::new();
    }
    let mut messages = Vec::new();
    match name {
        // An empty name is a missing one wearing quotes: no child can carry
        // `slot=""` past `check_contract`, so it names no declared slot either.
        OutletName::Literal(literal) if literal.is_empty() => {
            messages.push(MISSING_NAME_MESSAGE.to_owned());
        }
        OutletName::Literal(_) => {}
        OutletName::Dynamic => messages.push(DYNAMIC_NAME_MESSAGE.to_owned()),
        OutletName::Missing => messages.push(MISSING_NAME_MESSAGE.to_owned()),
    }
    for attribute in &outlet.extra_attributes {
        messages.push(extra_attribute_message(attribute));
    }
    if outlet.has_children {
        messages.push(OUTLET_CHILDREN_MESSAGE.to_owned());
    }
    messages
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bare() -> OutletUse<'static> {
        OutletUse::default()
    }

    #[test]
    fn a_named_outlet_is_accepted() {
        assert!(check_outlet("slot", OutletName::Literal("header"), &bare()).is_empty());
    }

    #[test]
    fn an_unnamed_outlet_is_refused() {
        assert_eq!(
            check_outlet("slot", OutletName::Missing, &bare()),
            [MISSING_NAME_MESSAGE]
        );
    }

    #[test]
    fn an_empty_name_is_refused_as_a_missing_one() {
        assert_eq!(
            check_outlet("slot", OutletName::Literal(""), &bare()),
            [MISSING_NAME_MESSAGE]
        );
    }

    #[test]
    fn a_dynamic_name_is_refused() {
        assert_eq!(
            check_outlet("slot", OutletName::Dynamic, &bare()),
            [DYNAMIC_NAME_MESSAGE]
        );
    }

    #[test]
    fn any_other_attribute_is_refused_and_named() {
        let outlet = OutletUse {
            extra_attributes: vec!["class", "on:click"],
            has_children: false,
        };
        let messages = check_outlet("slot", OutletName::Literal("header"), &outlet);
        assert_eq!(messages.len(), 2);
        assert!(messages[0].contains("'class'"));
        assert!(messages[1].contains("'on:click'"));
    }

    #[test]
    fn children_are_refused() {
        let outlet = OutletUse {
            extra_attributes: Vec::new(),
            has_children: true,
        };
        assert_eq!(
            check_outlet("slot", OutletName::Literal("header"), &outlet),
            [OUTLET_CHILDREN_MESSAGE]
        );
    }

    #[test]
    fn every_violation_is_reported_not_just_the_first() {
        let outlet = OutletUse {
            extra_attributes: vec!["class"],
            has_children: true,
        };
        assert_eq!(check_outlet("slot", OutletName::Missing, &outlet).len(), 3);
    }

    #[test]
    fn an_ordinary_tag_is_not_this_rules_business() {
        // The disjoint half. `name` is an ordinary prop on any other tag — as
        // `attr_symbol_on` agrees — so none of these rules may reach one.
        let outlet = OutletUse {
            extra_attributes: vec!["class"],
            has_children: true,
        };
        for tag in ["div", "Card", "Button", "Slot", "For", "If"] {
            assert!(
                check_outlet(tag, OutletName::Missing, &outlet).is_empty(),
                "{tag}"
            );
        }
    }

    #[test]
    fn the_rule_and_the_binding_name_the_same_tag() {
        // A `check_outlet` that policed a different tag from the one
        // `name_binding` lowers would police nothing.
        assert!(is_outlet_tag(OUTLET_TAG));
        assert!(!is_outlet_tag("Slot"));
    }

    #[test]
    fn each_message_names_the_construct_and_the_way_out() {
        // The author has to learn two things from one line: what is wrong, and
        // what to write instead.
        assert!(MISSING_NAME_MESSAGE.contains("`<slot>`"));
        assert!(MISSING_NAME_MESSAGE.contains("name=\"…\""));
        assert!(DYNAMIC_NAME_MESSAGE.contains("name=\"…\""));
        assert!(OUTLET_CHILDREN_MESSAGE.contains("`<slot>`"));
        assert!(extra_attribute_message("class").contains("'class'"));
    }
}
