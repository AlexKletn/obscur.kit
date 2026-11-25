#![doc = include_str!("../../docs/api/validation/contract_rule.doc/_module.md")]

use crate::validation::{ChildPolicy, ComponentContract, SlotRule};

pub use super::{
    AttributeUse, ContractViolation, ElementUse, MemberArity, MemberUse, SlotUse, ViolationAnchor,
};

#[doc = include_str!("../../docs/api/validation/contract_rule.doc/RESERVED_ATTRIBUTES.md")]
const RESERVED_ATTRIBUTES: [&str; 6] = ["key", "class", "id", "requires", "slot", "style"];

/// Returns whether `name` is a reserved structural attribute rather than a prop.
#[must_use]
pub fn is_reserved_attribute(name: &str) -> bool {
    RESERVED_ATTRIBUTES.contains(&name)
}

#[doc = include_str!("../../docs/api/validation/contract_rule.doc/check_contract.md")]
#[must_use]
pub fn check_contract(
    element: &ElementUse<'_>,
    contract: &ComponentContract,
) -> Vec<ContractViolation> {
    let mut violations = Vec::new();
    check_props(element, contract, &mut violations);
    check_children(element, contract, &mut violations);
    check_slots(element, contract, &mut violations);
    violations
}

/// Appends the required-prop, unknown-attribute and unknown-event violations.
fn check_props(
    element: &ElementUse<'_>,
    contract: &ComponentContract,
    violations: &mut Vec<ContractViolation>,
) {
    for prop in contract.required_props() {
        let present = element
            .attributes
            .iter()
            .any(|a| !a.event && a.name == prop.as_ref());
        if !present {
            violations.push(ContractViolation::MissingProp {
                prop: prop.clone(),
                anchor: ViolationAnchor::Element,
            });
        }
    }

    let restrict_props =
        !contract.required_props().is_empty() || !contract.allowed_props().is_empty();
    let restrict_events = !contract.allowed_events().is_empty();
    for (index, attribute) in element.attributes.iter().enumerate() {
        let anchor = ViolationAnchor::Attribute(index);
        if attribute.event {
            if restrict_events && !contract.permits_event(attribute.name) {
                violations.push(ContractViolation::UnknownEvent {
                    event: attribute.name.into(),
                    anchor,
                });
            }
        } else if restrict_props && !contract.permits_prop(attribute.name) {
            violations.push(ContractViolation::UnknownAttribute {
                name: attribute.name.into(),
                anchor,
            });
        }
    }
}

#[doc = include_str!("../../docs/api/validation/contract_rule.doc/check_children.md")]
fn check_children(
    element: &ElementUse<'_>,
    contract: &ComponentContract,
    violations: &mut Vec<ContractViolation>,
) {
    let policy = contract.children();
    if matches!(policy, ChildPolicy::Any) {
        return;
    }
    for (index, member) in element.members.iter().enumerate() {
        if policy.admits(member.tag) {
            continue;
        }
        let anchor = ViolationAnchor::Member(index);
        let tag: Box<str> = member.tag.into();
        violations.push(if policy.is_none() {
            ContractViolation::ForbiddenChild { tag, anchor }
        } else {
            ContractViolation::DisallowedChild { tag, anchor }
        });
    }
    if element.content && !policy.admits_content() {
        violations.push(ContractViolation::ForbiddenContent {
            // The instance itself: neither front end records where a text child
            // sits (the members carry the positions, and content is not a member),
            // and pointing at the tag the author wrote is the honest resolution.
            anchor: ViolationAnchor::Element,
        });
    }
}

/// Appends the §2.12 slot violations: assignment, membership and cardinality.
fn check_slots(
    element: &ElementUse<'_>,
    contract: &ComponentContract,
    violations: &mut Vec<ContractViolation>,
) {
    for (index, member) in element.members.iter().enumerate() {
        let Some(assignment) = member.slot else {
            continue;
        };
        let anchor = ViolationAnchor::Member(index);
        let SlotUse::Named(name) = assignment else {
            violations.push(ContractViolation::DynamicSlot { anchor });
            continue;
        };
        if contract.slots().is_empty() {
            violations.push(ContractViolation::NoSuchSlotSurface {
                slot: name.into(),
                anchor,
            });
            continue;
        }
        let Some(slot) = contract.slot(name) else {
            violations.push(ContractViolation::UnknownSlot {
                slot: name.into(),
                anchor,
            });
            continue;
        };
        if !slot.admits(member.tag) {
            violations.push(ContractViolation::DisallowedSlotChild {
                slot: name.into(),
                tag: member.tag.into(),
                anchor,
            });
        }
    }

    for slot in contract.slots() {
        check_cardinality(element, slot, violations);
    }
}

#[doc = include_str!("../../docs/api/validation/contract_rule.doc/check_cardinality.md")]
fn check_cardinality(
    element: &ElementUse<'_>,
    slot: &SlotRule,
    violations: &mut Vec<ContractViolation>,
) {
    let assigned: Vec<MemberArity> = element
        .members
        .iter()
        .filter(|m| m.slot == Some(SlotUse::Named(slot.name())))
        .map(|m| m.arity)
        .collect();
    let certain = assigned.iter().filter(|a| **a == MemberArity::Once).count();
    let max = if assigned.contains(&MemberArity::Any) {
        usize::MAX
    } else {
        assigned.len()
    };

    let cardinality = slot.cardinality();
    if (certain.min(2)..=max.min(2)).any(|count| cardinality.accepts(count)) {
        return;
    }
    let anchor = ViolationAnchor::Element;
    if certain > 1 {
        violations.push(ContractViolation::SlotOverfull {
            slot: slot.name().into(),
            cardinality,
            found: certain,
            anchor,
        });
    } else {
        violations.push(ContractViolation::SlotEmpty {
            slot: slot.name().into(),
            anchor,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_view::contract::SlotCardinality;

    fn card() -> ComponentContract {
        ComponentContract::new("Card")
            .with_slot(SlotRule::new("header", SlotCardinality::ExactlyOne))
            .with_slot(SlotRule::new("body", SlotCardinality::Many))
            .with_slot(SlotRule::new("footer", SlotCardinality::Optional).allowing_child("Button"))
    }

    fn member<'a>(tag: &'a str, slot: Option<&'a str>, arity: MemberArity) -> MemberUse<'a> {
        MemberUse {
            tag,
            slot: slot.map(SlotUse::Named),
            arity,
        }
    }

    fn use_of(members: Vec<MemberUse<'_>>) -> ElementUse<'_> {
        ElementUse {
            tag: "Card",
            attributes: Vec::new(),
            members,
            content: false,
        }
    }

    #[test]
    fn a_well_formed_instance_has_no_violations() {
        let element = use_of(vec![
            member("h1", Some("header"), MemberArity::Once),
            member("p", Some("body"), MemberArity::Once),
            member("p", Some("body"), MemberArity::Once),
        ]);
        assert_eq!(check_contract(&element, &card()), Vec::new());
    }

    #[test]
    fn an_undeclared_slot_name_is_a_violation() {
        let element = use_of(vec![member("h1", Some("headr"), MemberArity::Once)]);
        let violations = check_contract(&element, &card());
        assert!(
            violations.iter().any(|v| matches!(
                v,
                ContractViolation::UnknownSlot { slot, .. } if slot.as_ref() == "headr"
            )),
            "{violations:?}"
        );
    }

    #[test]
    fn a_slot_on_a_component_with_none_declared_names_nothing() {
        let element = ElementUse {
            tag: "Button",
            attributes: Vec::new(),
            members: vec![member("h1", Some("header"), MemberArity::Once)],
            content: false,
        };
        let violations = check_contract(&element, &ComponentContract::new("Button"));
        assert!(
            matches!(
                violations.as_slice(),
                [ContractViolation::NoSuchSlotSurface { .. }]
            ),
            "{violations:?}"
        );
    }

    #[test]
    fn exactly_one_rejects_two_certain_children_and_none_at_all() {
        let two = use_of(vec![
            member("h1", Some("header"), MemberArity::Once),
            member("h2", Some("header"), MemberArity::Once),
        ]);
        assert!(
            check_contract(&two, &card())
                .iter()
                .any(|v| matches!(v, ContractViolation::SlotOverfull { found: 2, .. })),
            "two certain children over-fill an ExactlyOne slot"
        );

        let none = use_of(vec![member("p", Some("body"), MemberArity::Once)]);
        assert!(
            check_contract(&none, &card())
                .iter()
                .any(|v| matches!(v, ContractViolation::SlotEmpty { .. })),
            "an unfilled ExactlyOne slot is a violation"
        );
    }

    #[test]
    fn a_conditional_member_is_never_accused_of_a_count_it_may_not_reach() {
        // One `<If>`-guarded header: the slot may hold 0 or 1, and both satisfy
        // nothing-is-certainly-wrong. Reporting either bound would be a false
        // error on code that is fine.
        let element = use_of(vec![member("h1", Some("header"), MemberArity::AtMostOnce)]);
        assert_eq!(check_contract(&element, &card()), Vec::new());

        // Two of them: still no *certain* member, so still nothing to report.
        let two = use_of(vec![
            member("h1", Some("header"), MemberArity::AtMostOnce),
            member("h2", Some("header"), MemberArity::AtMostOnce),
        ]);
        assert_eq!(check_contract(&two, &card()), Vec::new());
    }

    #[test]
    fn a_loop_body_leaves_an_optional_slot_unbounded_but_a_certain_pair_does_not() {
        let looped = use_of(vec![
            member("h1", Some("header"), MemberArity::Once),
            member("Button", Some("footer"), MemberArity::Any),
        ]);
        assert_eq!(
            check_contract(&looped, &card()),
            Vec::new(),
            "a `<For>` body may iterate once; nothing is certainly wrong"
        );

        let certain = use_of(vec![
            member("h1", Some("header"), MemberArity::Once),
            member("Button", Some("footer"), MemberArity::Once),
            member("Button", Some("footer"), MemberArity::Once),
        ]);
        assert!(
            check_contract(&certain, &card()).iter().any(|v| matches!(
                v,
                ContractViolation::SlotOverfull {
                    cardinality: SlotCardinality::Optional,
                    found: 2,
                    ..
                }
            )),
            "two certain children over-fill an Optional slot"
        );
    }

    #[test]
    fn a_slot_restricts_its_own_membership() {
        let element = use_of(vec![
            member("h1", Some("header"), MemberArity::Once),
            member("Input", Some("footer"), MemberArity::Once),
        ]);
        let violations = check_contract(&element, &card());
        assert!(
            violations.iter().any(|v| matches!(
                v,
                ContractViolation::DisallowedSlotChild { slot, tag, .. }
                    if slot.as_ref() == "footer" && tag.as_ref() == "Input"
            )),
            "{violations:?}"
        );
    }

    #[test]
    fn slot_is_reserved_so_it_is_never_the_childs_own_unknown_prop() {
        let element = ElementUse {
            tag: "Button",
            attributes: vec![
                AttributeUse {
                    name: "label",
                    event: false,
                },
                AttributeUse {
                    name: "slot",
                    event: false,
                },
            ],
            members: Vec::new(),
            content: false,
        };
        let contract = ComponentContract::new("Button").requiring_prop("label");
        assert_eq!(check_contract(&element, &contract), Vec::new());
    }

    #[test]
    fn a_runtime_slot_name_cannot_be_checked_and_says_so() {
        let element = use_of(vec![MemberUse {
            tag: "h1",
            slot: Some(SlotUse::Dynamic),
            arity: MemberArity::Once,
        }]);
        let violations = check_contract(&element, &card());
        assert!(
            violations
                .iter()
                .any(|v| matches!(v, ContractViolation::DynamicSlot { .. })),
            "{violations:?}"
        );
    }

    #[test]
    fn a_contract_that_takes_no_children_refuses_an_element_and_text_alike() {
        // The `<Button>OK</Button>` case: the text has no tag, so only a claim
        // about the *position* can catch it.
        let contract = ComponentContract::new("Button")
            .requiring_prop("label")
            .no_children();
        let text = ElementUse {
            tag: "Button",
            attributes: vec![AttributeUse {
                name: "label",
                event: false,
            }],
            members: Vec::new(),
            content: true,
        };
        assert!(
            matches!(
                check_contract(&text, &contract).as_slice(),
                [ContractViolation::ForbiddenContent { .. }]
            ),
            "{:?}",
            check_contract(&text, &contract)
        );

        let element = ElementUse {
            tag: "Button",
            attributes: vec![AttributeUse {
                name: "label",
                event: false,
            }],
            members: vec![member("span", None, MemberArity::Once)],
            content: false,
        };
        let violations = check_contract(&element, &contract);
        assert!(
            matches!(
                violations.as_slice(),
                [ContractViolation::ForbiddenChild { tag, .. }] if tag.as_ref() == "span"
            ),
            "{violations:?}"
        );
        assert_eq!(
            violations[0].message("Button"),
            "component 'Button' takes no children, so the 'span' written inside it \
             can never render",
            "the diagnostic names the tag and says what is wrong"
        );
    }

    #[test]
    fn a_forbidden_child_is_reported_even_behind_a_conditional() {
        // Unlike a cardinality bound, this is not a claim about a count: the
        // markup's only possible effect is nothing, whether the branch is taken or
        // not.
        let contract = ComponentContract::new("Button").no_children();
        let element = ElementUse {
            tag: "Button",
            attributes: Vec::new(),
            members: vec![member("span", None, MemberArity::AtMostOnce)],
            content: false,
        };
        assert!(
            check_contract(&element, &contract)
                .iter()
                .any(|v| matches!(v, ContractViolation::ForbiddenChild { .. })),
        );
    }

    #[test]
    fn a_tag_list_restricts_tags_and_leaves_content_alone() {
        // `Only` is a list of names, and a text run has none — so the two child
        // states report differently, which is why they are two violations.
        let contract = ComponentContract::new("Tabs").allowing_child("Tab");
        let element = ElementUse {
            tag: "Tabs",
            attributes: Vec::new(),
            members: vec![member("Tab", None, MemberArity::Once)],
            content: true,
        };
        assert_eq!(check_contract(&element, &contract), Vec::new());
    }

    #[test]
    fn content_under_an_unrestricted_contract_is_still_fine() {
        // The regression guard for every ordinary `<div>text</div>`: recording
        // content must not make an open contract start refusing it.
        let mut element = use_of(vec![member("h1", Some("header"), MemberArity::Once)]);
        element.content = true;
        assert_eq!(check_contract(&element, &card()), Vec::new());
    }

    #[test]
    fn violations_anchor_to_the_part_that_caused_them() {
        let element = ElementUse {
            tag: "Card",
            attributes: vec![AttributeUse {
                name: "bogus",
                event: false,
            }],
            members: vec![member("h1", Some("nope"), MemberArity::Once)],
            content: false,
        };
        let contract = ComponentContract::new("Card")
            .allowing_prop("title")
            .with_slot(SlotRule::new("header", SlotCardinality::Many));
        let violations = check_contract(&element, &contract);
        assert!(
            violations
                .iter()
                .any(|v| v.anchor() == ViolationAnchor::Attribute(0)),
            "{violations:?}"
        );
        assert!(
            violations
                .iter()
                .any(|v| v.anchor() == ViolationAnchor::Member(0)),
            "{violations:?}"
        );
    }
}
