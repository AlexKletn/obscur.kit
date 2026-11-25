#[doc = include_str!("../../docs/api/validation/contract_rule.doc/ViolationAnchor.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationAnchor {
    /// The element instance as a whole.
    Element,
    /// The attribute at this index of [`ElementUse::attributes`](super::ElementUse::attributes).
    Attribute(usize),
    /// The member at this index of [`ElementUse::members`](super::ElementUse::members).
    Member(usize),
}
