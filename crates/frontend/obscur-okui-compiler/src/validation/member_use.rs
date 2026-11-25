use super::{MemberArity, SlotUse};

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
