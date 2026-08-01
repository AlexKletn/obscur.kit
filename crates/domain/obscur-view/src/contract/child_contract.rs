#![doc = include_str!("child_contract.doc/_module.md")]

use obscur_identity::ComponentId;

#[doc = include_str!("child_contract.doc/ChildContract.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChildContract {
    #[doc = include_str!("child_contract.doc/Any.md")]
    Any,
    #[doc = include_str!("child_contract.doc/None.md")]
    None,
    #[doc = include_str!("child_contract.doc/Only.md")]
    Only(Vec<ComponentId>),
}

impl ChildContract {
    #[doc = include_str!("child_contract.doc/allows.md")]
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
