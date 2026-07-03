//! Intent to create a new window (spec §6.1, §6.2).

use obscur_identity::WindowId;

use crate::window::{Window, WindowConstraints, WindowProperties, WindowRelation};

/// An application's intent to create a new window (spec §6.1, §6.2).
///
/// A `CreateWindow` bundles the desired [`WindowProperties`], size
/// [`WindowConstraints`], and [`WindowRelation`] for a window that does not yet
/// exist. It is a pure command value: a backend consumes it, mints a
/// [`WindowId`], and the request can then be materialized into a retained
/// [`Window`] via [`CreateWindow::into_window`].
#[derive(Debug, Clone, PartialEq)]
pub struct CreateWindow {
    /// The desired presentation properties of the new window.
    properties: WindowProperties,
    /// The size constraints to apply to the new window.
    constraints: WindowConstraints,
    /// The owner/modality relationship of the new window to others.
    relation: WindowRelation,
}

impl CreateWindow {
    /// Creates a request for a window with the given properties.
    ///
    /// Constraints default to unconstrained and the relation to a top-level,
    /// modeless window.
    #[must_use]
    pub fn new(properties: WindowProperties) -> Self {
        Self {
            properties,
            constraints: WindowConstraints::UNCONSTRAINED,
            relation: WindowRelation::top_level(),
        }
    }

    /// Sets the size constraints, returning the updated request.
    #[must_use]
    pub fn with_constraints(mut self, constraints: WindowConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    /// Sets the owner/modality relation, returning the updated request.
    #[must_use]
    pub fn with_relation(mut self, relation: WindowRelation) -> Self {
        self.relation = relation;
        self
    }

    /// Returns a shared reference to the requested properties.
    #[must_use]
    pub const fn properties(&self) -> &WindowProperties {
        &self.properties
    }

    /// Returns the requested size constraints.
    #[must_use]
    pub const fn constraints(&self) -> WindowConstraints {
        self.constraints
    }

    /// Returns the requested owner/modality relation.
    #[must_use]
    pub const fn relation(&self) -> WindowRelation {
        self.relation
    }

    /// Materializes this request into a retained [`Window`] using the identity
    /// the backend assigned.
    ///
    /// The requested constraints and relation are applied on top of the freshly
    /// constructed window, so the result reflects the full request.
    #[must_use]
    pub fn into_window(self, id: WindowId) -> Window {
        let mut window = Window::new(id, self.properties);
        window.set_constraints(self.constraints);
        window.set_relation(self.relation);
        window
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::window::Modality;
    use obscur_geometry::Size;

    #[test]
    fn new_uses_defaults() {
        let req = CreateWindow::new(WindowProperties::default());
        assert_eq!(req.constraints(), WindowConstraints::UNCONSTRAINED);
        assert_eq!(req.relation(), WindowRelation::top_level());
        assert_eq!(req.properties().size, Size::new(800.0, 600.0));
    }

    #[test]
    fn builders_set_fields() {
        let constraints = WindowConstraints::with_max(Size::new(400.0, 300.0));
        let relation = WindowRelation::owned_by(WindowId::from_raw(9))
            .with_modality(Modality::WindowModal);
        let req = CreateWindow::new(WindowProperties::default())
            .with_constraints(constraints)
            .with_relation(relation);

        assert_eq!(req.constraints(), constraints);
        assert_eq!(req.relation(), relation);
    }

    #[test]
    fn into_window_applies_constraints_and_relation() {
        let constraints = WindowConstraints::with_max(Size::new(400.0, 300.0));
        let relation = WindowRelation::owned_by(WindowId::from_raw(2));
        let req = CreateWindow::new(WindowProperties::default().with_title("Main"))
            .with_constraints(constraints)
            .with_relation(relation);

        let window = req.into_window(WindowId::from_raw(5));

        assert_eq!(window.id(), WindowId::from_raw(5));
        assert_eq!(&*window.properties().title, "Main");
        assert_eq!(window.constraints(), constraints);
        assert_eq!(window.relation(), relation);
    }
}
