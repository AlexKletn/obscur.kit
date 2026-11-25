//! Owner/parent relations and modality between windows (spec §6.1).

use obscur_identity::WindowId;

use super::Modality;

#[doc = include_str!("../../docs/api/window/window_relation.doc/WindowRelation.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowRelation {
    /// The owning/parent window, or `None` for a top-level window.
    pub owner: Option<WindowId>,
    /// How this window restricts interaction with other windows.
    pub modality: Modality,
}

impl WindowRelation {
    /// A top-level, modeless window with no owner.
    #[must_use]
    pub const fn top_level() -> Self {
        Self {
            owner: None,
            modality: Modality::Modeless,
        }
    }

    /// A modeless window owned by `owner`.
    #[must_use]
    pub const fn owned_by(owner: WindowId) -> Self {
        Self {
            owner: Some(owner),
            modality: Modality::Modeless,
        }
    }

    /// Sets the owner window, returning the updated relation.
    #[must_use]
    pub const fn with_owner(mut self, owner: WindowId) -> Self {
        self.owner = Some(owner);
        self
    }

    /// Sets the modality, returning the updated relation.
    #[must_use]
    pub const fn with_modality(mut self, modality: Modality) -> Self {
        self.modality = modality;
        self
    }

    /// Returns whether this window is top-level (has no owner).
    #[must_use]
    pub const fn is_top_level(&self) -> bool {
        self.owner.is_none()
    }
}

impl Default for WindowRelation {
    /// The default relation is a top-level, modeless window.
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
