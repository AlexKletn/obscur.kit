/// A member's slot assignment, owned. The borrowed
/// [`SlotUse`](crate::validation::contract_rule::SlotUse) the rule takes cannot
/// hold a `String` the AST just minted.
pub(super) enum Slot {
    /// `slot="header"`.
    Named(String),
    /// `slot={expr}`.
    Dynamic,
}
