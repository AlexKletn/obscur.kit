//! The §2.12 outlet rule: what may be written on an authored `<slot …/>`, stated
//! over plain data so **both** front ends decide it the same way.
//!
//! # What an outlet is, and what that costs it
//!
//! `<slot name="header"/>` is a *position*, not an element:
//! `project_slots` deletes it and splices in every authored child assigned
//! `slot="header"`. Nothing an outlet carries survives that substitution — the
//! node the author wrote is gone before style, layout or paint can see it.
//!
//! That is what makes every rule below a rule rather than a preference. A
//! `class="banner"` on an outlet is not "ignored"; it is *deleted*, silently, and
//! the author is left reading a stylesheet that never matched anything. §2.13 says
//! unknown attributes are compile errors; on an outlet, *every* attribute but
//! `name` is unknown, because there is no node left to carry it.
//!
//! # The rules
//!
//! - **`name` is required.** An outlet with no name names no slot, so
//!   `project_slots` fills it with nothing and deletes it. It is a slot
//!   declaration that can never be satisfied — exactly the "inert box" this
//!   refuses at the tag instead of at the frame.
//! - **`name` must be a literal.** `name={expr}` is refused for the reason
//!   `slot={expr}` already is
//!   ([`DynamicSlot`](crate::validation::ContractViolation::DynamicSlot)): a slot
//!   is matched against a declaration at *compile* time, and a name that does not
//!   exist until runtime cannot be. The two halves of one join must be decidable
//!   in the same pass, or neither is.
//! - **Nothing else may be written on it.** Not `class`, not `id`, not `on:click`,
//!   not `key` — see above.
//! - **It takes no children.** HTML's `<slot>` renders its children as fallback
//!   content when nothing is assigned. §2.12 describes no fallback content, and
//!   [`project_slots`] implements none: an unfilled outlet leaves *nothing*
//!   behind. Accepting children would mean silently dropping them. Inventing
//!   fallback semantics because HTML has them is the move `5e8e7fe` refused for
//!   `<Loading>`; refusing the spelling is what this does instead.
//!
//! # One rule, two front ends
//!
//! The rule takes plain data and returns messages, with no syntax in it, for the
//! same reason [`check_collection`](crate::validation::check_collection) and
//! [`check_contract`](crate::validation::check_contract) do: there are two
//! authoring surfaces and only one has a `.okui` syntax tree. Each front end
//! attaches its own location — a byte offset for the file path, a `Span` for the
//! `okui!` macro.
//!
//! # What this rule cannot decide
//!
//! Whether the outlet is in a **component's render at all**. A component's `render`
//! returns a `View` and `okui!` produces a `View`, so the identical
//! `okui! { <slot name="header"/> }` is correct inside `render` and stray one
//! function away, and a proc-macro cannot see which of the two it was expanded in.
//! Nor can it check the name against the component's declared slots: the
//! declaration lives on `MyProps::contract_surface()`, a function that runs when
//! the built crate runs, while this runs during that crate's compilation — the same
//! wall that keeps an unregistered `<Crad/>` silent. The stray case is decided at
//! runtime instead, where the answer is knowable: the reconciler refuses to
//! materialize an outlet and `UiRuntime` reports it.
//!
//! [`project_slots`]: obscur_view::view::project_slots

use obscur_ui_ir::name_binding::OUTLET_TAG;

/// What an authored `<slot …/>` names its outlet — the value of its `name`
/// attribute, classified (spec §2.12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutletName<'a> {
    /// `name="header"` — a literal, and the only accepted form.
    Literal(&'a str),
    /// `name={expr}` — a name that does not exist until runtime.
    Dynamic,
    /// No `name` attribute at all.
    Missing,
}

/// One authored `<slot …/>`, described in the shape both front ends can build
/// from their own syntax (spec §2.12).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OutletUse<'a> {
    /// Every attribute name on the tag *except* `name`, in authored order —
    /// including event bindings, spelled as the author wrote the event
    /// (`on:click` → `"on:click"`), so the message quotes them back.
    pub extra_attributes: Vec<&'a str>,
    /// Whether the tag has any children (elements, text or interpolations).
    pub has_children: bool,
}

/// Whether `tag` names a §2.12 slot outlet.
///
/// One predicate over [`OUTLET_TAG`], so the rule below and the front ends that
/// decide whether to run it cannot disagree about which tag it is.
#[must_use]
pub fn is_outlet_tag(tag: &str) -> bool {
    tag == OUTLET_TAG
}

/// The diagnostic for a `<slot/>` with no `name` (spec §2.12).
pub const MISSING_NAME_MESSAGE: &str = "`<slot>` requires a `name`: §2.12 declares *named* slots and projection matches an outlet to content by name and by nothing else, so an unnamed outlet is filled by nothing and deleted. Write `<slot name=\"…\"/>`";

/// The diagnostic for a `<slot name={expr}/>` (spec §2.12).
pub const DYNAMIC_NAME_MESSAGE: &str = "`<slot name={…}>` is refused: a slot is matched against its component's declaration at compile time, so an outlet's name must be a literal — the same rule that refuses `slot={…}` on the child side of the same join. Write `<slot name=\"…\"/>`";

/// The diagnostic for children under a `<slot>` (spec §2.12).
pub const OUTLET_CHILDREN_MESSAGE: &str = "`<slot>` takes no children: an outlet is substituted away by projection and an unfilled one leaves nothing behind, so children here would be dropped without a trace. §2.12 describes no fallback content and none is implemented";

/// The diagnostic for any attribute other than `name` on a `<slot>` (spec §2.12).
#[must_use]
pub fn extra_attribute_message(attribute: &str) -> String {
    format!(
        "unknown attribute '{attribute}' on `<slot>`: an outlet is a position, not an element — projection deletes the node before style, layout or paint can read it, so it carries nothing but its `name`"
    )
}

/// Checks an authored `<slot …/>` against the §2.12 outlet rule, returning one
/// message per violation.
///
/// Empty for every tag that is not an outlet, and for a well-formed
/// `<slot name="…"/>`. Every violation is reported rather than just the first,
/// the same report-them-all policy `Compiler::compile` and the `okui!` macro's
/// `validate_element` follow.
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
