//! The retained domain record of a live window (spec §6.1, §6.2).

use obscur_geometry::{ScaleFactor, Size};
use obscur_identity::WindowId;

use crate::window::{WindowConstraints, WindowProperties, WindowRelation};

/// The retained domain record of a live window (spec §6.1, §6.2).
///
/// A `Window` bundles a window's stable identity with its desired
/// [`WindowProperties`], its size [`WindowConstraints`], its
/// [`WindowRelation`] to other windows, and its current display scale factor.
/// It is a pure data record; a backend owns the real platform window and
/// reconciles it against this model.
#[derive(Debug, Clone, PartialEq)]
pub struct Window {
    /// The stable identity of this window.
    id: WindowId,
    /// The desired presentation properties.
    properties: WindowProperties,
    /// The size constraints applied to the window.
    constraints: WindowConstraints,
    /// The owner/modality relationship to other windows.
    relation: WindowRelation,
    /// The current display scale factor.
    scale_factor: ScaleFactor,
}

impl Window {
    /// Creates a window from an identity and desired properties.
    ///
    /// Constraints default to unconstrained, the relation to a top-level
    /// modeless window, and the scale factor to the identity (1.0).
    #[must_use]
    pub fn new(id: WindowId, properties: WindowProperties) -> Self {
        Self {
            id,
            properties,
            constraints: WindowConstraints::UNCONSTRAINED,
            relation: WindowRelation::top_level(),
            scale_factor: ScaleFactor::IDENTITY,
        }
    }

    /// Returns the window's identity.
    #[must_use]
    pub const fn id(&self) -> WindowId {
        self.id
    }

    /// Returns a shared reference to the window's properties.
    #[must_use]
    pub const fn properties(&self) -> &WindowProperties {
        &self.properties
    }

    /// Returns a mutable reference to the window's properties.
    pub const fn properties_mut(&mut self) -> &mut WindowProperties {
        &mut self.properties
    }

    /// Returns the window's size constraints.
    #[must_use]
    pub const fn constraints(&self) -> WindowConstraints {
        self.constraints
    }

    /// Sets the window's size constraints.
    pub const fn set_constraints(&mut self, constraints: WindowConstraints) {
        self.constraints = constraints;
    }

    /// Returns the window's relation to other windows.
    #[must_use]
    pub const fn relation(&self) -> WindowRelation {
        self.relation
    }

    /// Sets the window's relation to other windows.
    pub const fn set_relation(&mut self, relation: WindowRelation) {
        self.relation = relation;
    }

    /// Returns the window's current display scale factor.
    #[must_use]
    pub const fn scale_factor(&self) -> ScaleFactor {
        self.scale_factor
    }

    /// Sets the window's current display scale factor.
    pub const fn set_scale_factor(&mut self, scale_factor: ScaleFactor) {
        self.scale_factor = scale_factor;
    }

    /// Returns the window's desired size clamped by its constraints.
    #[must_use]
    pub fn constrained_size(&self) -> Size {
        self.constraints.constrain(self.properties.size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::window::Modality;

    fn window() -> Window {
        Window::new(WindowId::from_raw(1), WindowProperties::default())
    }

    #[test]
    fn new_uses_defaults() {
        let w = window();
        assert_eq!(w.id(), WindowId::from_raw(1));
        assert_eq!(w.constraints(), WindowConstraints::UNCONSTRAINED);
        assert_eq!(w.relation(), WindowRelation::top_level());
        assert_eq!(w.scale_factor(), ScaleFactor::IDENTITY);
        assert_eq!(w.properties().size, Size::new(800.0, 600.0));
    }

    #[test]
    fn properties_mut_edits_in_place() {
        let mut w = window();
        w.properties_mut().title = Box::from("Main");
        assert_eq!(&*w.properties().title, "Main");
    }

    #[test]
    fn setters_update_fields() {
        let mut w = window();
        w.set_constraints(WindowConstraints::with_max(Size::new(400.0, 300.0)));
        w.set_relation(WindowRelation::owned_by(WindowId::from_raw(9)).with_modality(Modality::WindowModal));
        w.set_scale_factor(ScaleFactor::new(2.0));

        assert_eq!(w.relation().owner, Some(WindowId::from_raw(9)));
        assert_eq!(w.relation().modality, Modality::WindowModal);
        assert_eq!(w.scale_factor(), ScaleFactor::new(2.0));
    }

    #[test]
    fn constrained_size_applies_constraints() {
        let mut w = window();
        w.set_constraints(WindowConstraints::with_max(Size::new(400.0, 300.0)));
        assert_eq!(w.constrained_size(), Size::new(400.0, 300.0));
    }
}
