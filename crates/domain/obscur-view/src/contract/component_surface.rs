//! `ComponentSurface`: the string-keyed *tooling* view of a component's usage
//! contract (spec §2.13).

use crate::contract::SlotSurface;

use super::{ChildrenSurface, PropTypeSurface};

#[doc = include_str!("../../docs/api/contract/component_surface.doc/ComponentSurface.md")]
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
    /// Creates a surface for `component` with no required props and no prop,
    /// event or child restriction (children default to [`ChildrenSurface::Any`]).
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

    #[doc = include_str!("../../docs/api/contract/component_surface.doc/with_prop_type.md")]
    #[must_use]
    pub fn with_prop_type(mut self, name: &'static str, ty: PropTypeSurface) -> Self {
        self.prop_types.push((name, ty));
        self
    }

    /// Returns the declared type category of prop `name`, if one was recorded.
    #[must_use]
    pub fn prop_type(&self, name: &str) -> Option<PropTypeSurface> {
        self.prop_types
            .iter()
            .find(|(prop, _)| *prop == name)
            .map(|(_, ty)| *ty)
    }

    /// Returns every recorded `(prop name, type)` pair, in declaration order.
    #[must_use]
    pub fn prop_types(&self) -> &[(&'static str, PropTypeSurface)] {
        &self.prop_types
    }

    /// Adds `name` to the surface's required props (fluent).
    #[must_use]
    pub fn requiring_prop(mut self, name: &'static str) -> Self {
        self.required_props.push(name);
        self
    }

    /// Adds `name` to the surface's allowed (optional) props (fluent).
    #[must_use]
    pub fn allowing_prop(mut self, name: &'static str) -> Self {
        self.allowed_props.push(name);
        self
    }

    /// Adds `name` to the surface's allowed `on:<event>` names (fluent).
    #[must_use]
    pub fn allowing_event(mut self, name: &'static str) -> Self {
        self.allowed_events.push(name);
        self
    }

    /// Adds `tag` to the surface's allowed child tags (fluent), switching the
    /// children rule to [`ChildrenSurface::Only`].
    #[must_use]
    pub fn allowing_child(mut self, tag: &'static str) -> Self {
        match &mut self.children {
            ChildrenSurface::Only(tags) => tags.push(tag),
            _ => self.children = ChildrenSurface::Only(vec![tag]),
        }
        self
    }

    /// Sets the children rule to [`ChildrenSurface::Any`] (fluent).
    #[must_use]
    pub fn any_children(mut self) -> Self {
        self.children = ChildrenSurface::Any;
        self
    }

    /// Sets the children rule to [`ChildrenSurface::None`] (a leaf; fluent).
    #[must_use]
    pub fn no_children(mut self) -> Self {
        self.children = ChildrenSurface::None;
        self
    }

    /// Declares a named slot on the surface (fluent; spec §2.12).
    #[must_use]
    pub fn with_slot(mut self, slot: SlotSurface) -> Self {
        self.slots.push(slot);
        self
    }

    /// Returns the described component's name.
    #[must_use]
    pub fn component(&self) -> &str {
        self.component
    }

    /// Returns the required prop names.
    #[must_use]
    pub fn required_props(&self) -> &[&'static str] {
        &self.required_props
    }

    /// Returns the allowed (optional) prop names.
    #[must_use]
    pub fn allowed_props(&self) -> &[&'static str] {
        &self.allowed_props
    }

    /// Returns the allowed `on:<event>` names.
    #[must_use]
    pub fn allowed_events(&self) -> &[&'static str] {
        &self.allowed_events
    }

    /// Returns the children rule.
    #[must_use]
    pub fn children(&self) -> &ChildrenSurface {
        &self.children
    }

    /// Returns the declared named slots (spec §2.12); an empty slice means the
    /// component declares none, which *closes* the slot surface rather than
    /// leaving it open — the asymmetry with props/children is deliberate, since
    /// a `slot="…"` naming nothing has no meaning to fall back to.
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
}
