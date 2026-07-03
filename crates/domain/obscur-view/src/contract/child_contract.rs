//! `ChildContract`: which children a slot or component accepts (spec §2.12).

use obscur_identity::ComponentId;

/// The rule describing which child components are allowed in a given position
/// (a slot, or a component's direct children) per spec §2.12.
///
/// This is a small, typed allow-list rather than a string matcher: unknown
/// children are rejected at contract-check time. The whitelist variant carries
/// the concrete [`ComponentId`]s that are permitted, so membership is decided
/// by identity, not by name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChildContract {
    /// Any child component is accepted.
    Any,
    /// No children are accepted (a leaf position).
    None,
    /// Only components whose id appears in this list are accepted.
    Only(Vec<ComponentId>),
}

impl ChildContract {
    /// Returns whether a child with the given `id` is allowed by this contract.
    #[must_use]
    pub fn allows(&self, id: ComponentId) -> bool {
        match self {
            Self::Any => true,
            Self::None => false,
            Self::Only(allowed) => allowed.contains(&id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any_allows_all_and_none_allows_nothing() {
        let id = ComponentId::from_raw(7);
        assert!(ChildContract::Any.allows(id));
        assert!(!ChildContract::None.allows(id));
    }

    #[test]
    fn whitelist_checks_membership() {
        let allowed = ComponentId::from_raw(7);
        let other = ComponentId::from_raw(8);
        let contract = ChildContract::Only(vec![allowed]);
        assert!(contract.allows(allowed));
        assert!(!contract.allows(other));
    }
}
