use obscur_view::contract::SlotCardinality;

use super::ViolationAnchor;

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
    #[doc = include_str!("../../docs/api/validation/contract_rule.doc/ForbiddenChild.md")]
    ForbiddenChild {
        /// The offending child's tag.
        tag: Box<str>,
        /// Where to report it.
        anchor: ViolationAnchor,
    },
    /// Text or `{expr}` content written inside a component that admits **no**
    /// children.
    ForbiddenContent {
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
            | Self::ForbiddenChild { anchor, .. }
            | Self::ForbiddenContent { anchor }
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
            Self::ForbiddenChild { tag, .. } => format!(
                "component '{component}' takes no children, so the '{tag}' written \
                 inside it can never render"
            ),
            Self::ForbiddenContent { .. } => format!(
                "component '{component}' takes no children, so the content written \
                 inside it can never render"
            ),
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
