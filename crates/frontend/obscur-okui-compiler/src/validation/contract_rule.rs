//! The §2.13/§2.12 contract rule itself, stated over plain data so **both**
//! front ends decide a document's legality the same way.
//!
//! # Why the rule is not stated over the AST
//!
//! There are two authoring surfaces and only one of them has a `.okui` syntax
//! tree. The file path parses to [`Element`](obscur_okui_syntax::ast::Element)
//! nodes; the `okui!` macro parses `TokenStream`s with `syn` and never builds
//! one. A rule written against either input can only ever check that input —
//! which is exactly how the macro came to run **no** contract validation at all
//! while the file path ran this one over every element it parsed.
//!
//! So the rule takes an [`ElementUse`]: the tag, the attribute names, and the
//! members, with no syntax in it. Each front end describes its own instance and
//! attaches its own location to the resulting [`ContractViolation`]s — a byte
//! offset for the file path, a `Span` for the macro. One rule, two adapters:
//! the same shape `check_expression` already gave §2.10.
//!
//! # Members, not children
//!
//! A component's *members* are the nodes that actually land in its child list,
//! which is not the same as its direct children: `<If>` and `<For>` splice their
//! bodies into the enclosing sink (see `ViewCodegen::emit_into`), so
//! `<Card><If cond={x}><h1 slot="header"/></If></Card>` puts the `h1` in Card's
//! children, not the `If`'s. The rule therefore looks through control flow — and
//! records what that costs it: a member behind an `<If>` may not be there, and a
//! member inside a `<For>` may be there many times. [`MemberArity`] carries that,
//! and cardinality reports only what is *certainly* wrong.

use obscur_view::contract::SlotCardinality;

use crate::validation::{ComponentContract, SlotRule};

/// How many nodes one member of an [`ElementUse`] contributes to its parent's
/// child list.
///
/// A static check cannot count a conditional, so it does not pretend to: the
/// arity is the honest bound, and [`check_contract`] only reports a cardinality
/// violation that holds for *every* value the bound admits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemberArity {
    /// Exactly one node: an unconditional child.
    Once,
    /// Zero or one node: the member sits under an `<If>`/`<ElseIf>`/`<Else>`.
    AtMostOnce,
    /// Any number of nodes: the member sits inside a `<For>` body.
    Any,
}

/// One attribute written on an element instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttributeUse<'a> {
    /// The attribute's name — for an event, the local name after `on:`.
    pub name: &'a str,
    /// Whether this is an `on:<event>` binding rather than a prop.
    pub event: bool,
}

/// What a member's `slot="…"` attribute assigns it to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotUse<'a> {
    /// A literal slot name, which is the only kind a contract can be checked
    /// against.
    Named(&'a str),
    /// `slot={expr}`. A slot is matched against a declaration at *compile* time,
    /// so a name that only exists at runtime cannot be checked against one — and
    /// nothing downstream would route it either.
    Dynamic,
}

/// One member of an element instance's child list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemberUse<'a> {
    /// The member's tag name.
    pub tag: &'a str,
    /// The slot the member was assigned to, from its `slot="…"` attribute.
    pub slot: Option<SlotUse<'a>>,
    /// How many nodes this member contributes (see [`MemberArity`]).
    pub arity: MemberArity,
}

/// One element instance, described with no syntax in it, ready to be checked
/// against a [`ComponentContract`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementUse<'a> {
    /// The instance's tag name.
    pub tag: &'a str,
    /// The attributes written on it, in source order.
    pub attributes: Vec<AttributeUse<'a>>,
    /// Its members, in source order (see the module docs).
    pub members: Vec<MemberUse<'a>>,
}

/// Where a [`ContractViolation`] should be reported, as an index into the
/// [`ElementUse`] that produced it.
///
/// The rule does not know what a location *is* — the file path has byte offsets
/// and the macro has spans — so it names the offending part and lets the caller
/// resolve it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationAnchor {
    /// The element instance as a whole.
    Element,
    /// The attribute at this index of [`ElementUse::attributes`].
    Attribute(usize),
    /// The member at this index of [`ElementUse::members`].
    Member(usize),
}

/// One way an element instance breaks its component's contract (spec §2.12,
/// §2.13).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractViolation {
    /// A required prop is not set.
    MissingProp {
        /// The absent prop's name.
        prop: Box<str>,
        /// Where to report it.
        anchor: ViolationAnchor,
    },
    /// An attribute the contract's closed prop surface does not declare.
    UnknownAttribute {
        /// The offending attribute's name.
        name: Box<str>,
        /// Where to report it.
        anchor: ViolationAnchor,
    },
    /// An `on:<event>` the contract's closed event surface does not declare.
    UnknownEvent {
        /// The offending event's name.
        event: Box<str>,
        /// Where to report it.
        anchor: ViolationAnchor,
    },
    /// A child tag the contract's child restriction does not allow.
    DisallowedChild {
        /// The offending child's tag.
        tag: Box<str>,
        /// Where to report it.
        anchor: ViolationAnchor,
    },
    /// A `slot="…"` on a child of a component that declares no slots at all.
    NoSuchSlotSurface {
        /// The name the child asked for.
        slot: Box<str>,
        /// Where to report it.
        anchor: ViolationAnchor,
    },
    /// A `slot="…"` naming a slot the component does not declare.
    UnknownSlot {
        /// The name the child asked for.
        slot: Box<str>,
        /// Where to report it.
        anchor: ViolationAnchor,
    },
    /// A child tag the slot's own child restriction does not allow.
    DisallowedSlotChild {
        /// The slot the child was assigned to.
        slot: Box<str>,
        /// The offending child's tag.
        tag: Box<str>,
        /// Where to report it.
        anchor: ViolationAnchor,
    },
    /// More children are assigned to a slot than its cardinality accepts.
    SlotOverfull {
        /// The over-subscribed slot.
        slot: Box<str>,
        /// The cardinality it declared.
        cardinality: SlotCardinality,
        /// How many children are certainly in it.
        found: usize,
        /// Where to report it.
        anchor: ViolationAnchor,
    },
    /// A slot declaring [`SlotCardinality::ExactlyOne`] has no child at all.
    SlotEmpty {
        /// The unfilled slot.
        slot: Box<str>,
        /// Where to report it.
        anchor: ViolationAnchor,
    },
    /// A `slot={expr}` — a slot name that does not exist until runtime.
    DynamicSlot {
        /// Where to report it.
        anchor: ViolationAnchor,
    },
}

impl ContractViolation {
    /// Returns where this violation should be reported.
    #[must_use]
    pub const fn anchor(&self) -> ViolationAnchor {
        match self {
            Self::MissingProp { anchor, .. }
            | Self::UnknownAttribute { anchor, .. }
            | Self::UnknownEvent { anchor, .. }
            | Self::DisallowedChild { anchor, .. }
            | Self::NoSuchSlotSurface { anchor, .. }
            | Self::UnknownSlot { anchor, .. }
            | Self::DisallowedSlotChild { anchor, .. }
            | Self::SlotOverfull { anchor, .. }
            | Self::SlotEmpty { anchor, .. }
            | Self::DynamicSlot { anchor } => *anchor,
        }
    }

    /// Renders the diagnostic message for this violation on `component`.
    #[must_use]
    pub fn message(&self, component: &str) -> String {
        match self {
            Self::MissingProp { prop, .. } => {
                format!("component '{component}' requires prop '{prop}'")
            }
            Self::UnknownAttribute { name, .. } => {
                format!("unknown attribute '{name}' on component '{component}'")
            }
            Self::UnknownEvent { event, .. } => {
                format!("unknown event '{event}' on component '{component}'")
            }
            Self::DisallowedChild { tag, .. } => {
                format!("'{tag}' is not an allowed child of '{component}'")
            }
            Self::NoSuchSlotSurface { slot, .. } => {
                format!(
                    "component '{component}' declares no slots, so 'slot=\"{slot}\"' names nothing"
                )
            }
            Self::UnknownSlot { slot, .. } => {
                format!("unknown slot '{slot}' on component '{component}'")
            }
            Self::DisallowedSlotChild { slot, tag, .. } => {
                format!(
                    "'{tag}' is not an allowed child of slot '{slot}' on component '{component}'"
                )
            }
            Self::SlotOverfull {
                slot,
                cardinality,
                found,
                ..
            } => {
                let limit = match cardinality {
                    SlotCardinality::ExactlyOne => "exactly one child",
                    SlotCardinality::Optional => "at most one child",
                    SlotCardinality::Many => "any number of children",
                };
                format!("slot '{slot}' of component '{component}' accepts {limit}, found {found}")
            }
            Self::SlotEmpty { slot, .. } => {
                format!("component '{component}' requires exactly one child in slot '{slot}'")
            }
            Self::DynamicSlot { .. } => format!(
                "a slot assignment on '{component}' must be a literal name, not an expression"
            ),
        }
    }
}

/// Attribute names always permitted on any element whatever its contract's
/// declared prop surface: the structural keys, the `requires="cap"` capability
/// declaration (§2.13), and `slot` — a child's slot assignment (§2.12), which is
/// addressed to its *parent*'s contract and so is never the child's own prop.
const RESERVED_ATTRIBUTES: [&str; 5] = ["key", "class", "id", "requires", "slot"];

/// Returns whether `name` is a reserved structural attribute rather than a prop.
#[must_use]
pub fn is_reserved_attribute(name: &str) -> bool {
    RESERVED_ATTRIBUTES.contains(&name)
}

/// Checks one element instance against `contract`, returning every violation it
/// commits (spec §2.12, §2.13).
///
/// The checks, in order:
///
/// - **required props** — every required prop must appear as an attribute;
/// - **unknown attributes** — when the prop surface is closed, every non-event
///   attribute must be declared or [reserved](is_reserved_attribute);
/// - **unknown events** — when the event surface is closed, every `on:<event>`
///   must be declared;
/// - **allowed children** — when the contract restricts children, every member's
///   tag must be listed;
/// - **slots** (§2.12) — every member's `slot="…"` must name a declared slot and
///   satisfy that slot's own child restriction, and each slot's
///   [`SlotCardinality`] must hold.
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

/// Appends the allowed-children violations.
fn check_children(
    element: &ElementUse<'_>,
    contract: &ComponentContract,
    violations: &mut Vec<ContractViolation>,
) {
    if contract.allowed_children().is_empty() {
        return;
    }
    for (index, member) in element.members.iter().enumerate() {
        if !contract
            .allowed_children()
            .iter()
            .any(|allowed| allowed.as_ref() == member.tag)
        {
            violations.push(ContractViolation::DisallowedChild {
                tag: member.tag.into(),
                anchor: ViolationAnchor::Member(index),
            });
        }
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

/// Appends the cardinality violation for one slot, if the bound proves one.
///
/// The number of children a slot ends up with is not a number but a **range**:
/// `certain` (the members that are definitely there) up to `max` (all of them,
/// or unbounded once a `<For>` body is among them). A violation exists only when
/// [`SlotCardinality::accepts`] rejects *every* count in that range — otherwise
/// the author has written something that may well be right, and a compile error
/// would be a lie.
///
/// The range is probed up to 2 because that is all a cardinality distinguishes:
/// none, one, more than one.
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
    fn violations_anchor_to_the_part_that_caused_them() {
        let element = ElementUse {
            tag: "Card",
            attributes: vec![AttributeUse {
                name: "bogus",
                event: false,
            }],
            members: vec![member("h1", Some("nope"), MemberArity::Once)],
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
