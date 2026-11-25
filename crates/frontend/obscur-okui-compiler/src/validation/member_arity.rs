#[doc = include_str!("../../docs/api/validation/contract_rule.doc/MemberArity.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemberArity {
    /// Exactly one node: an unconditional child.
    Once,
    /// Zero or one node: the member sits under an `<If>`/`<ElseIf>`/`<Else>`.
    AtMostOnce,
    /// Any number of nodes: the member sits inside a `<For>` body.
    Any,
}
