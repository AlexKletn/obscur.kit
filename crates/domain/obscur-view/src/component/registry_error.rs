//! `RegistryError`: typed failures from [`ComponentRegistry`](crate::component::ComponentRegistry)
//! operations.

use obscur_identity::ComponentId;

#[doc = include_str!("../../docs/api/component/registry_error.doc/RegistryError.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RegistryError {
    /// A registration was rejected because its [`ComponentId`] is already
    /// registered. Carries the conflicting id.
    DuplicateComponent(ComponentId),
}

impl core::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::DuplicateComponent(id) => {
                write!(f, "component {id:?} is already registered")
            }
        }
    }
}

impl core::error::Error for RegistryError {}

#[cfg(test)]
mod tests {
    use super::RegistryError;
    use obscur_identity::ComponentId;

    #[test]
    fn display_names_the_conflicting_id() {
        let err = RegistryError::DuplicateComponent(ComponentId::from_raw(4));
        assert!(format!("{err}").contains("already registered"));
    }

    #[test]
    fn is_comparable() {
        assert_eq!(
            RegistryError::DuplicateComponent(ComponentId::from_raw(1)),
            RegistryError::DuplicateComponent(ComponentId::from_raw(1))
        );
        assert_ne!(
            RegistryError::DuplicateComponent(ComponentId::from_raw(1)),
            RegistryError::DuplicateComponent(ComponentId::from_raw(2))
        );
    }
}
