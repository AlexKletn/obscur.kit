#![doc = include_str!("window_relation.doc/_module.md")]

use obscur_identity::WindowId;

#[doc = include_str!("window_relation.doc/Modality.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modality {
    #[doc = include_str!("window_relation.doc/Modeless.md")]
    Modeless,
    #[doc = include_str!("window_relation.doc/WindowModal.md")]
    WindowModal,
    #[doc = include_str!("window_relation.doc/ApplicationModal.md")]
    ApplicationModal,
}

impl Modality {
    #[doc = include_str!("window_relation.doc/is_modal.md")]
    #[must_use]
    pub const fn is_modal(self) -> bool {
        !matches!(self, Self::Modeless)
    }
}

impl Default for Modality {
    #[doc = include_str!("window_relation.doc/default.md")]
    fn default() -> Self {
        Self::Modeless
    }
}

#[doc = include_str!("window_relation.doc/WindowRelation.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowRelation {
    #[doc = include_str!("window_relation.doc/owner.md")]
    pub owner: Option<WindowId>,
    #[doc = include_str!("window_relation.doc/modality.md")]
    pub modality: Modality,
}

impl WindowRelation {
    #[doc = include_str!("window_relation.doc/top_level.md")]
    #[must_use]
    pub const fn top_level() -> Self {
        Self {
            owner: None,
            modality: Modality::Modeless,
        }
    }

    #[doc = include_str!("window_relation.doc/owned_by.md")]
    #[must_use]
    pub const fn owned_by(owner: WindowId) -> Self {
        Self {
            owner: Some(owner),
            modality: Modality::Modeless,
        }
    }

    #[doc = include_str!("window_relation.doc/with_owner.md")]
    #[must_use]
    pub const fn with_owner(mut self, owner: WindowId) -> Self {
        self.owner = Some(owner);
        self
    }

    #[doc = include_str!("window_relation.doc/with_modality.md")]
    #[must_use]
    pub const fn with_modality(mut self, modality: Modality) -> Self {
        self.modality = modality;
        self
    }

    #[doc = include_str!("window_relation.doc/is_top_level.md")]
    #[must_use]
    pub const fn is_top_level(&self) -> bool {
        self.owner.is_none()
    }
}

impl Default for WindowRelation {
    #[doc = include_str!("window_relation.doc/default_2.md")]
    fn default() -> Self {
        Self::top_level()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_top_level_modeless() {
        let r = WindowRelation::default();
        assert_eq!(r, WindowRelation::top_level());
        assert!(r.is_top_level());
        assert_eq!(r.modality, Modality::Modeless);
        assert!(!r.modality.is_modal());
    }

    #[test]
    fn owned_by_sets_owner() {
        let owner = WindowId::from_raw(7);
        let r = WindowRelation::owned_by(owner);
        assert_eq!(r.owner, Some(owner));
        assert!(!r.is_top_level());
    }

    #[test]
    fn setters_chain() {
        let owner = WindowId::from_raw(3);
        let r = WindowRelation::top_level()
            .with_owner(owner)
            .with_modality(Modality::ApplicationModal);
        assert_eq!(r.owner, Some(owner));
        assert_eq!(r.modality, Modality::ApplicationModal);
        assert!(r.modality.is_modal());
    }
}
