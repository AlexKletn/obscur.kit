//! `ViewNode`: the core recursive node of the desired view tree.

use obscur_identity::ComponentId;

use super::{ViewEventBinding, ViewFragment, ViewKey, ViewProperty};

/// A single node in a component's desired declarative view tree.
///
/// The desired tree is *plain data*: cloneable, comparable, and free of any
/// retained-tree references (`NodeId`), runtime handles, or closures. Updates
/// are expressed by producing a new desired tree rather than mutating a
/// retained one (spec §2.2). Keyed identity used for reconciliation lives here
/// as an optional [`ViewKey`]; meaningful structure and text are both real
/// nodes (spec §4.16).
///
/// Because element props carry [`super::ViewValue`] (which contains an `f64`),
/// this enum derives only `Debug`, `Clone`, and `PartialEq`.
#[derive(Debug, Clone, PartialEq)]
pub enum ViewNode {
    /// A component element: a keyed, propertied node with event bindings and
    /// recursively nested children.
    Element {
        /// Stable id of the component this element renders (spec §2.15).
        component: ComponentId,
        /// Optional explicit key for keyed reconciliation.
        key: Option<ViewKey>,
        /// Properties flowed *down* into the element.
        props: Vec<ViewProperty>,
        /// Event bindings whose actions flow *up*.
        events: Vec<ViewEventBinding>,
        /// Recursively nested children.
        children: ViewFragment,
    },
    /// A text node — text is real structure in the tree (spec §4.16).
    Text {
        /// Optional explicit key for keyed reconciliation.
        key: Option<ViewKey>,
        /// The literal text content.
        content: Box<str>,
    },
}

impl ViewNode {
    /// Creates an unkeyed element node for `component` with no props, events,
    /// or children.
    #[must_use]
    pub fn element(component: ComponentId) -> Self {
        Self::Element {
            component,
            key: None,
            props: Vec::new(),
            events: Vec::new(),
            children: ViewFragment::new(),
        }
    }

    /// Creates a **slot outlet**: the position in a component's own output where
    /// the children an author assigned to the slot called `name` are projected
    /// (spec §2.12).
    ///
    /// This is the half of a slot that a component owns. The other half is the
    /// author's `<h1 slot="header"/>`; [`project_slots`](super::project_slots)
    /// joins them, replacing this node with that content before the output is
    /// reconciled — so an outlet never becomes a retained node, filled or not.
    ///
    /// `name` must be one of the slots the component's
    /// [`ComponentContract`](crate::component::ComponentContract) declares: that
    /// declaration is what both front ends check an author's assignment against,
    /// so an outlet naming an undeclared slot can never receive anything.
    #[must_use]
    pub fn slot(name: impl Into<Box<str>>) -> Self {
        Self::element(super::SLOT_OUTLET).with_prop(ViewProperty::new(
            super::SLOT,
            super::ViewValue::Text(name.into()),
        ))
    }

    /// Creates a **virtual-collection marker**: the node a `<VirtualFor>` lowers
    /// to, carrying the per-row `template` as its children and the estimated row
    /// `row_extent` (logical pixels along the scroll axis) as a reserved property
    /// (spec §2.11, §4.9).
    ///
    /// This is the half of a `<VirtualFor>` the front end can build: the window
    /// it stands for is a function of the viewport, which the runtime owns, so
    /// [`realize_virtual_collections`](super::realize_virtual_collections)
    /// replaces this marker with a viewport-bounded window of `template` clones
    /// before the output is reconciled — the marker itself never becomes a
    /// retained node, the same way a slot outlet never does.
    #[must_use]
    pub fn virtual_collection(row_extent: f32, template: ViewFragment) -> Self {
        Self::element(super::VIRTUAL_COLLECTION)
            .with_prop(ViewProperty::new(
                super::VIRTUAL_ROW_EXTENT,
                super::ViewValue::Float(f64::from(row_extent)),
            ))
            .with_children(template)
    }

    /// Creates an unkeyed text node holding `content`.
    #[must_use]
    pub fn text(content: impl Into<Box<str>>) -> Self {
        Self::Text {
            key: None,
            content: content.into(),
        }
    }

    /// Returns this node's explicit key, if any.
    #[must_use]
    pub fn key(&self) -> Option<&ViewKey> {
        match self {
            Self::Element { key, .. } | Self::Text { key, .. } => key.as_ref(),
        }
    }

    /// Sets this node's explicit key, returning the updated node.
    #[must_use]
    pub fn with_key(mut self, new_key: impl Into<ViewKey>) -> Self {
        let new_key = new_key.into();
        match &mut self {
            Self::Element { key, .. } | Self::Text { key, .. } => *key = Some(new_key),
        }
        self
    }

    /// Appends a property to an element node. No-op on a text node.
    #[must_use]
    pub fn with_prop(mut self, prop: ViewProperty) -> Self {
        if let Self::Element { props, .. } = &mut self {
            props.push(prop);
        }
        self
    }

    /// Appends an event binding to an element node. No-op on a text node.
    #[must_use]
    pub fn with_event(mut self, event: ViewEventBinding) -> Self {
        if let Self::Element { events, .. } = &mut self {
            events.push(event);
        }
        self
    }

    /// Sets the children of an element node. No-op on a text node.
    #[must_use]
    pub fn with_children(mut self, new_children: ViewFragment) -> Self {
        if let Self::Element { children, .. } = &mut self {
            *children = new_children;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::ViewValue;
    use obscur_identity::InternedSymbol;

    #[test]
    fn element_starts_empty_and_unkeyed() {
        let node = ViewNode::element(ComponentId::from_raw(1));
        assert!(node.key().is_none());
        match node {
            ViewNode::Element {
                props,
                events,
                children,
                ..
            } => {
                assert!(props.is_empty());
                assert!(events.is_empty());
                assert!(children.is_empty());
            }
            ViewNode::Text { .. } => panic!("expected element"),
        }
    }

    #[test]
    fn text_holds_content() {
        let node = ViewNode::text("hello");
        match &node {
            ViewNode::Text { content, .. } => assert_eq!(content.as_ref(), "hello"),
            ViewNode::Element { .. } => panic!("expected text"),
        }
    }

    #[test]
    fn with_key_sets_key_on_both_variants() {
        let el = ViewNode::element(ComponentId::from_raw(1)).with_key(2u64);
        assert_eq!(el.key(), Some(&ViewKey::from(2u64)));
        let tx = ViewNode::text("x").with_key("id");
        assert_eq!(tx.key(), Some(&ViewKey::from("id")));
    }

    #[test]
    fn with_prop_appends_on_element_only() {
        let prop = ViewProperty::new(InternedSymbol::from_raw(1), ViewValue::Bool(true));
        let el = ViewNode::element(ComponentId::from_raw(1)).with_prop(prop.clone());
        match el {
            ViewNode::Element { props, .. } => assert_eq!(props, vec![prop.clone()]),
            ViewNode::Text { .. } => panic!("expected element"),
        }
        // No-op path on text.
        let tx = ViewNode::text("x").with_prop(prop);
        assert!(matches!(tx, ViewNode::Text { .. }));
    }
}
