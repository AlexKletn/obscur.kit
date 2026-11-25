//! `ViewNode`: the core recursive node of the desired view tree.

use obscur_identity::{ComponentId, InternedSymbol};

use super::{CLASS, STYLE, ViewEventBinding, ViewFragment, ViewKey, ViewProperty, ViewValue};

#[doc = include_str!("../../docs/api/view/view_node.doc/ViewNode.md")]
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

    #[doc = include_str!("../../docs/api/view/view_node.doc/slot.md")]
    #[must_use]
    pub fn slot(name: impl Into<Box<str>>) -> Self {
        Self::element(super::SLOT_OUTLET).with_prop(ViewProperty::new(
            super::SLOT,
            super::ViewValue::Text(name.into()),
        ))
    }

    #[doc = include_str!("../../docs/api/view/view_node.doc/virtual_collection.md")]
    #[must_use]
    pub fn virtual_collection(row_extent: f32, template: ViewFragment) -> Self {
        Self::element(super::VIRTUAL_COLLECTION)
            .with_prop(ViewProperty::new(
                super::VIRTUAL_ROW_EXTENT,
                super::ViewValue::Float(f64::from(row_extent)),
            ))
            .with_children(template)
    }

    #[doc = include_str!("../../docs/api/view/view_node.doc/virtual_list.md")]
    #[must_use]
    pub fn virtual_list(list_id: InternedSymbol, count: usize, row_extent: f32) -> Self {
        let count_value = match i64::try_from(count) {
            Ok(count) => super::ViewValue::Int(count),
            Err(_) => super::ViewValue::Text(count.to_string().into_boxed_str()),
        };
        Self::element(super::VIRTUAL_COLLECTION)
            .with_prop(ViewProperty::new(
                super::VIRTUAL_LIST_ID,
                super::ViewValue::Int(i64::from(list_id.raw())),
            ))
            .with_prop(ViewProperty::new(super::VIRTUAL_ITEM_COUNT, count_value))
            .with_prop(ViewProperty::new(
                super::VIRTUAL_ROW_EXTENT,
                super::ViewValue::Float(f64::from(row_extent)),
            ))
    }

    /// Creates a virtual-list marker with an owned row builder. The runtime calls the
    /// builder only for indices in the realized window. The list ID identifies the
    /// scroll window, while keys on the returned nodes identify the items.
    #[must_use]
    pub fn virtual_list_with_rows(
        list_id: InternedSymbol,
        count: usize,
        row_extent: f32,
        build: impl Fn(usize) -> ViewFragment + Send + Sync + 'static,
    ) -> Self {
        let mut marker = Self::virtual_list(list_id, count, row_extent);
        if let Self::Element { props, .. } = &mut marker
            && let Some(count) = props
                .iter_mut()
                .find(|prop| prop.name() == super::VIRTUAL_ITEM_COUNT)
        {
            *count = count
                .clone()
                .with_virtual_rows(super::virtual_row_builder::VirtualRowBuilder::new(build));
        }
        marker
    }

    /// Adds fixed-height content between adjacent items of an indexed virtual list.
    /// The extent is in logical pixels and defaults to one when omitted or invalid.
    /// The virtual layout accounts for separators outside the realized window.
    #[must_use]
    pub fn with_virtual_separator(self, extent: Option<f32>, content: ViewFragment) -> Self {
        let extent = extent
            .filter(|value| value.is_finite() && *value >= 0.0)
            .unwrap_or(1.0);
        self.with_prop(ViewProperty::new(
            super::virtual_collection::VIRTUAL_SEPARATOR_EXTENT,
            ViewValue::Float(f64::from(extent)),
        ))
        .with_children(content)
    }

    #[doc = include_str!("../../docs/api/view/view_node.doc/with_virtual_scroll_reset.md")]
    #[must_use]
    pub fn with_virtual_scroll_reset(self, token: u64) -> Self {
        #[expect(
            clippy::cast_possible_wrap,
            reason = "the token is opaque; the runtime compares its bit pattern, not its magnitude"
        )]
        let value = super::ViewValue::Int(token as i64);
        self.with_prop(ViewProperty::new(super::VIRTUAL_SCROLL_RESET, value))
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

    /// Marks a visual destination for a keyed shared-element transition.
    #[must_use]
    pub fn with_shared_element_key(mut self, key: impl Into<Box<str>>) -> Self {
        if let Self::Element { props, .. } = &mut self {
            props.retain(|prop| prop.name() != super::SHARED_ELEMENT_KEY);
            props.push(ViewProperty::new(
                super::SHARED_ELEMENT_KEY,
                ViewValue::Text(key.into()),
            ));
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

    #[doc = include_str!("../../docs/api/view/view_node.doc/class.md")]
    #[must_use]
    pub fn class(self, class: impl Into<Box<str>>) -> Self {
        self.with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
    }

    #[doc = include_str!("../../docs/api/view/view_node.doc/style.md")]
    #[must_use]
    pub fn style(self, declarations: impl Into<Box<str>>) -> Self {
        self.with_prop(ViewProperty::new(
            STYLE,
            ViewValue::Text(declarations.into()),
        ))
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

    #[test]
    fn class_sets_the_class_attribute_as_text() {
        let el = ViewNode::element(ComponentId::from_raw(1)).class("row sel");
        match el {
            ViewNode::Element { props, .. } => assert_eq!(
                props,
                vec![ViewProperty::new(CLASS, ViewValue::Text("row sel".into()))]
            ),
            ViewNode::Text { .. } => panic!("expected element"),
        }
    }

    #[test]
    fn class_is_a_noop_on_text() {
        let tx = ViewNode::text("x").class("row");
        assert!(matches!(tx, ViewNode::Text { .. }));
    }

    #[test]
    fn style_sets_the_reserved_style_symbol_not_a_hashed_one() {
        // The symbol is what decides whether the block is ever compiled:
        // `StyleAttributes::extract` reads `STYLE` and nothing else, so a prop
        // under any other symbol is carried and silently ignored.
        let el = ViewNode::element(ComponentId::from_raw(1)).style("width: 50%");
        match el {
            ViewNode::Element { props, .. } => assert_eq!(
                props,
                vec![ViewProperty::new(
                    STYLE,
                    ViewValue::Text("width: 50%".into())
                )]
            ),
            ViewNode::Text { .. } => panic!("expected element"),
        }
    }

    #[test]
    fn style_is_a_noop_on_text() {
        let tx = ViewNode::text("x").style("width: 50%");
        assert!(matches!(tx, ViewNode::Text { .. }));
    }
}
