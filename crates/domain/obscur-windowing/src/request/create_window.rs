#![doc = include_str!("create_window.doc/_module.md")]

use obscur_identity::WindowId;

use crate::window::{Window, WindowConstraints, WindowProperties, WindowRelation};

#[doc = include_str!("create_window.doc/CreateWindow.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct CreateWindow {
    #[doc = include_str!("create_window.doc/properties.md")]
    properties: WindowProperties,
    #[doc = include_str!("create_window.doc/constraints.md")]
    constraints: WindowConstraints,
    #[doc = include_str!("create_window.doc/relation.md")]
    relation: WindowRelation,
}

impl CreateWindow {
    #[doc = include_str!("create_window.doc/new.md")]
    #[must_use]
    pub fn new(properties: WindowProperties) -> Self {
        Self {
            properties,
            constraints: WindowConstraints::UNCONSTRAINED,
            relation: WindowRelation::top_level(),
        }
    }

    #[doc = include_str!("create_window.doc/with_constraints.md")]
    #[must_use]
    pub fn with_constraints(mut self, constraints: WindowConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    #[doc = include_str!("create_window.doc/with_relation.md")]
    #[must_use]
    pub fn with_relation(mut self, relation: WindowRelation) -> Self {
        self.relation = relation;
        self
    }

    #[doc = include_str!("create_window.doc/properties_2.md")]
    #[must_use]
    pub const fn properties(&self) -> &WindowProperties {
        &self.properties
    }

    #[doc = include_str!("create_window.doc/constraints_2.md")]
    #[must_use]
    pub const fn constraints(&self) -> WindowConstraints {
        self.constraints
    }

    #[doc = include_str!("create_window.doc/relation_2.md")]
    #[must_use]
    pub const fn relation(&self) -> WindowRelation {
        self.relation
    }

    #[doc = include_str!("create_window.doc/into_window.md")]
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
        let relation =
            WindowRelation::owned_by(WindowId::from_raw(9)).with_modality(Modality::WindowModal);
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
