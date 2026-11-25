use super::{AttributeUse, MemberUse};

/// One element instance, described with no syntax in it, ready to be checked
/// against a [`ComponentContract`](super::ComponentContract).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementUse<'a> {
    /// The instance's tag name.
    pub tag: &'a str,
    /// The attributes written on it, in source order.
    pub attributes: Vec<AttributeUse<'a>>,
    /// Its members, in source order (see the module docs).
    pub members: Vec<MemberUse<'a>>,
    #[doc = include_str!("../../docs/api/validation/contract_rule.doc/content.md")]
    pub content: bool,
}
