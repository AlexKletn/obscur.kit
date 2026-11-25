use obscur_view::contract::PropTypeSurface;

use super::{ChildPolicy, SlotRule, is_reserved_attribute};

#[doc = include_str!("../../docs/api/validation/contract_validator.doc/ComponentContract.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentContract {
    component: Box<str>,
    required_props: Vec<Box<str>>,
    allowed_props: Vec<Box<str>>,
    allowed_events: Vec<Box<str>>,
    children: ChildPolicy,
    prop_types: Vec<(Box<str>, PropTypeSurface)>,
    slots: Vec<SlotRule>,
}

impl ComponentContract {
    /// Creates a contract for `component` with no required props and no prop,
    /// event or child restriction (any attribute, event and child allowed).
    #[must_use]
    pub fn new(component: impl Into<Box<str>>) -> Self {
        Self {
            component: component.into(),
            required_props: Vec::new(),
            allowed_props: Vec::new(),
            allowed_events: Vec::new(),
            children: ChildPolicy::Any,
            prop_types: Vec::new(),
            slots: Vec::new(),
        }
    }

    #[doc = include_str!("../../docs/api/validation/contract_validator.doc/with_prop_type.md")]
    #[must_use]
    pub fn with_prop_type(mut self, name: impl Into<Box<str>>, ty: PropTypeSurface) -> Self {
        self.prop_types.push((name.into(), ty));
        self
    }

    /// Returns the declared type category of prop `name`, if one was recorded.
    #[must_use]
    pub fn prop_type(&self, name: &str) -> Option<PropTypeSurface> {
        self.prop_types
            .iter()
            .find(|(prop, _)| prop.as_ref() == name)
            .map(|(_, ty)| *ty)
    }

    /// Adds `name` to the contract's required props (fluent).
    #[must_use]
    pub fn requiring_prop(mut self, name: impl Into<Box<str>>) -> Self {
        self.required_props.push(name.into());
        self
    }

    #[doc = include_str!("../../docs/api/validation/contract_validator.doc/allowing_prop.md")]
    #[must_use]
    pub fn allowing_prop(mut self, name: impl Into<Box<str>>) -> Self {
        self.allowed_props.push(name.into());
        self
    }

    #[doc = include_str!("../../docs/api/validation/contract_validator.doc/allowing_event.md")]
    #[must_use]
    pub fn allowing_event(mut self, name: impl Into<Box<str>>) -> Self {
        self.allowed_events.push(name.into());
        self
    }

    #[doc = include_str!("../../docs/api/validation/contract_validator.doc/allowing_child.md")]
    #[must_use]
    pub fn allowing_child(mut self, tag: impl Into<Box<str>>) -> Self {
        self.children.allow(tag);
        self
    }

    #[doc = include_str!("../../docs/api/validation/contract_validator.doc/no_children.md")]
    #[must_use]
    pub fn no_children(mut self) -> Self {
        self.children = ChildPolicy::None;
        self
    }

    #[doc = include_str!("../../docs/api/validation/contract_validator.doc/with_slot.md")]
    #[must_use]
    pub fn with_slot(mut self, slot: SlotRule) -> Self {
        self.slots.push(slot);
        self
    }

    /// Returns the constrained component's name.
    #[must_use]
    pub fn component(&self) -> &str {
        &self.component
    }

    /// Returns the declared named slots (spec §2.12); an empty slice means the
    /// component declares none and accepts no slot assignment at all.
    #[must_use]
    pub fn slots(&self) -> &[SlotRule] {
        &self.slots
    }

    /// Returns the declared slot called `name`, if any.
    #[must_use]
    pub fn slot(&self, name: &str) -> Option<&SlotRule> {
        self.slots.iter().find(|slot| slot.name() == name)
    }

    /// Returns the required prop names.
    #[must_use]
    pub fn required_props(&self) -> &[Box<str>] {
        &self.required_props
    }

    /// Returns the allowed (optional) prop names. Props are unrestricted only
    /// when the contract declares no props at all (this slice *and*
    /// [`required_props`](Self::required_props) both empty).
    #[must_use]
    pub fn allowed_props(&self) -> &[Box<str>] {
        &self.allowed_props
    }

    /// Returns the allowed `on:<event>` names; an empty slice means events are
    /// unrestricted.
    #[must_use]
    pub fn allowed_events(&self) -> &[Box<str>] {
        &self.allowed_events
    }

    #[doc = include_str!("../../docs/api/validation/contract_validator.doc/children.md")]
    #[must_use]
    pub const fn children(&self) -> &ChildPolicy {
        &self.children
    }

    /// Returns whether `name` is an accepted attribute: a required prop, an
    /// allowed prop, or a reserved structural attribute. Only consulted when the
    /// prop surface is closed (`allowed_props` non-empty).
    pub(crate) fn permits_prop(&self, name: &str) -> bool {
        is_reserved_attribute(name)
            || self.required_props.iter().any(|p| p.as_ref() == name)
            || self.allowed_props.iter().any(|p| p.as_ref() == name)
    }

    /// Returns whether `event` is an accepted `on:<event>` name. Only consulted
    /// when the event surface is closed (`allowed_events` non-empty).
    pub(crate) fn permits_event(&self, event: &str) -> bool {
        self.allowed_events.iter().any(|e| e.as_ref() == event)
    }
}
