#![doc = include_str!("component_surface.doc/_module.md")]

use crate::contract::SlotSurface;

#[doc = include_str!("component_surface.doc/ChildrenSurface.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ChildrenSurface {
    #[doc = include_str!("component_surface.doc/Any.md")]
    #[default]
    Any,
    #[doc = include_str!("component_surface.doc/None.md")]
    None,
    #[doc = include_str!("component_surface.doc/Only.md")]
    Only(Vec<&'static str>),
}

#[doc = include_str!("component_surface.doc/PropTypeSurface.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PropTypeSurface {
    #[doc = include_str!("component_surface.doc/String.md")]
    String,
    #[doc = include_str!("component_surface.doc/Int.md")]
    Int,
    #[doc = include_str!("component_surface.doc/Float.md")]
    Float,
    #[doc = include_str!("component_surface.doc/Bool.md")]
    Bool,
    #[doc = include_str!("component_surface.doc/Other.md")]
    #[default]
    Other,
}

#[doc = include_str!("component_surface.doc/ComponentSurface.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ComponentSurface {
    component: &'static str,
    required_props: Vec<&'static str>,
    allowed_props: Vec<&'static str>,
    allowed_events: Vec<&'static str>,
    prop_types: Vec<(&'static str, PropTypeSurface)>,
    children: ChildrenSurface,
    slots: Vec<SlotSurface>,
}

impl ComponentSurface {
    #[doc = include_str!("component_surface.doc/new.md")]
    #[must_use]
    pub fn new(component: &'static str) -> Self {
        Self {
            component,
            required_props: Vec::new(),
            allowed_props: Vec::new(),
            allowed_events: Vec::new(),
            prop_types: Vec::new(),
            children: ChildrenSurface::Any,
            slots: Vec::new(),
        }
    }

    #[doc = include_str!("component_surface.doc/with_prop_type.md")]
    #[must_use]
    pub fn with_prop_type(mut self, name: &'static str, ty: PropTypeSurface) -> Self {
        self.prop_types.push((name, ty));
        self
    }

    #[doc = include_str!("component_surface.doc/prop_type.md")]
    #[must_use]
    pub fn prop_type(&self, name: &str) -> Option<PropTypeSurface> {
        self.prop_types
            .iter()
            .find(|(prop, _)| *prop == name)
            .map(|(_, ty)| *ty)
    }

    #[doc = include_str!("component_surface.doc/prop_types.md")]
    #[must_use]
    pub fn prop_types(&self) -> &[(&'static str, PropTypeSurface)] {
        &self.prop_types
    }

    #[doc = include_str!("component_surface.doc/requiring_prop.md")]
    #[must_use]
    pub fn requiring_prop(mut self, name: &'static str) -> Self {
        self.required_props.push(name);
        self
    }

    #[doc = include_str!("component_surface.doc/allowing_prop.md")]
    #[must_use]
    pub fn allowing_prop(mut self, name: &'static str) -> Self {
        self.allowed_props.push(name);
        self
    }

    #[doc = include_str!("component_surface.doc/allowing_event.md")]
    #[must_use]
    pub fn allowing_event(mut self, name: &'static str) -> Self {
        self.allowed_events.push(name);
        self
    }

    #[doc = include_str!("component_surface.doc/allowing_child.md")]
    #[must_use]
    pub fn allowing_child(mut self, tag: &'static str) -> Self {
        match &mut self.children {
            ChildrenSurface::Only(tags) => tags.push(tag),
            _ => self.children = ChildrenSurface::Only(vec![tag]),
        }
        self
    }

    #[doc = include_str!("component_surface.doc/any_children.md")]
    #[must_use]
    pub fn any_children(mut self) -> Self {
        self.children = ChildrenSurface::Any;
        self
    }

    #[doc = include_str!("component_surface.doc/no_children.md")]
    #[must_use]
    pub fn no_children(mut self) -> Self {
        self.children = ChildrenSurface::None;
        self
    }

    #[doc = include_str!("component_surface.doc/with_slot.md")]
    #[must_use]
    pub fn with_slot(mut self, slot: SlotSurface) -> Self {
        self.slots.push(slot);
        self
    }

    #[doc = include_str!("component_surface.doc/component.md")]
    #[must_use]
    pub fn component(&self) -> &str {
        self.component
    }

    #[doc = include_str!("component_surface.doc/required_props.md")]
    #[must_use]
    pub fn required_props(&self) -> &[&'static str] {
        &self.required_props
    }

    #[doc = include_str!("component_surface.doc/allowed_props.md")]
    #[must_use]
    pub fn allowed_props(&self) -> &[&'static str] {
        &self.allowed_props
    }

    #[doc = include_str!("component_surface.doc/allowed_events.md")]
    #[must_use]
    pub fn allowed_events(&self) -> &[&'static str] {
        &self.allowed_events
    }

    #[doc = include_str!("component_surface.doc/children.md")]
    #[must_use]
    pub fn children(&self) -> &ChildrenSurface {
        &self.children
    }

    #[doc = include_str!("component_surface.doc/slots.md")]
    #[must_use]
    pub fn slots(&self) -> &[SlotSurface] {
        &self.slots
    }
}

#[cfg(test)]
mod tests {
    use super::{ChildrenSurface, ComponentSurface};

    #[test]
    fn new_surface_is_open() {
        let surface = ComponentSurface::new("Box");
        assert_eq!(surface.component(), "Box");
        assert!(surface.required_props().is_empty());
        assert!(surface.allowed_props().is_empty());
        assert!(surface.allowed_events().is_empty());
        assert_eq!(surface.children(), &ChildrenSurface::Any);
    }

    #[test]
    fn fluent_builder_accumulates_every_facet() {
        let surface = ComponentSurface::new("Tabs")
            .requiring_prop("selected")
            .allowing_prop("orientation")
            .allowing_event("change")
            .allowing_child("Tab");
        assert_eq!(surface.required_props(), &["selected"]);
        assert_eq!(surface.allowed_props(), &["orientation"]);
        assert_eq!(surface.allowed_events(), &["change"]);
        assert_eq!(surface.children(), &ChildrenSurface::Only(vec!["Tab"]));
    }

    #[test]
    fn allowing_child_accumulates_a_tag_list() {
        let surface = ComponentSurface::new("List")
            .allowing_child("Item")
            .allowing_child("Divider");
        assert_eq!(
            surface.children(),
            &ChildrenSurface::Only(vec!["Item", "Divider"])
        );
    }

    #[test]
    fn any_and_no_children_set_the_rule() {
        assert_eq!(
            ComponentSurface::new("A").no_children().children(),
            &ChildrenSurface::None
        );
        assert_eq!(
            ComponentSurface::new("B")
                .allowing_child("X")
                .any_children()
                .children(),
            &ChildrenSurface::Any
        );
    }

    #[test]
    fn declared_slots_are_kept_in_order() {
        use crate::contract::{SlotCardinality, SlotSurface};

        let surface = ComponentSurface::new("Card")
            .with_slot(SlotSurface::new("header", SlotCardinality::ExactlyOne))
            .with_slot(SlotSurface::new("body", SlotCardinality::Many));
        let names: Vec<&str> = surface.slots().iter().map(SlotSurface::name).collect();
        assert_eq!(names, vec!["header", "body"]);
        assert!(ComponentSurface::new("Box").slots().is_empty());
    }

    #[test]
    fn default_children_surface_is_any() {
        assert_eq!(ChildrenSurface::default(), ChildrenSurface::Any);
    }
}
