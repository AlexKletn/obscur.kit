#![doc = include_str!("window.doc/_module.md")]

use obscur_geometry::{ScaleFactor, Size};
use obscur_identity::WindowId;

use crate::window::{WindowConstraints, WindowProperties, WindowRelation};

#[doc = include_str!("window.doc/Window.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Window {
    #[doc = include_str!("window.doc/id.md")]
    id: WindowId,
    #[doc = include_str!("window.doc/properties.md")]
    properties: WindowProperties,
    #[doc = include_str!("window.doc/constraints.md")]
    constraints: WindowConstraints,
    #[doc = include_str!("window.doc/relation.md")]
    relation: WindowRelation,
    #[doc = include_str!("window.doc/scale_factor.md")]
    scale_factor: ScaleFactor,
}

impl Window {
    #[doc = include_str!("window.doc/new.md")]
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

    #[doc = include_str!("window.doc/id_2.md")]
    #[must_use]
    pub const fn id(&self) -> WindowId {
        self.id
    }

    #[doc = include_str!("window.doc/properties_2.md")]
    #[must_use]
    pub const fn properties(&self) -> &WindowProperties {
        &self.properties
    }

    #[doc = include_str!("window.doc/properties_mut.md")]
    pub const fn properties_mut(&mut self) -> &mut WindowProperties {
        &mut self.properties
    }

    #[doc = include_str!("window.doc/constraints_2.md")]
    #[must_use]
    pub const fn constraints(&self) -> WindowConstraints {
        self.constraints
    }

    #[doc = include_str!("window.doc/set_constraints.md")]
    pub const fn set_constraints(&mut self, constraints: WindowConstraints) {
        self.constraints = constraints;
    }

    #[doc = include_str!("window.doc/relation_2.md")]
    #[must_use]
    pub const fn relation(&self) -> WindowRelation {
        self.relation
    }

    #[doc = include_str!("window.doc/set_relation.md")]
    pub const fn set_relation(&mut self, relation: WindowRelation) {
        self.relation = relation;
    }

    #[doc = include_str!("window.doc/scale_factor_2.md")]
    #[must_use]
    pub const fn scale_factor(&self) -> ScaleFactor {
        self.scale_factor
    }

    #[doc = include_str!("window.doc/set_scale_factor.md")]
    pub const fn set_scale_factor(&mut self, scale_factor: ScaleFactor) {
        self.scale_factor = scale_factor;
    }

    #[doc = include_str!("window.doc/constrained_size.md")]
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
        w.set_relation(
            WindowRelation::owned_by(WindowId::from_raw(9)).with_modality(Modality::WindowModal),
        );
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
