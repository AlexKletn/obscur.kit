#![doc = include_str!("view_node.doc/_module.md")]

use obscur_identity::{ComponentId, InternedSymbol};

use super::{CLASS, STYLE, ViewEventBinding, ViewFragment, ViewKey, ViewProperty, ViewValue};

#[doc = include_str!("view_node.doc/ViewNode.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum ViewNode {
    #[doc = include_str!("view_node.doc/Element.md")]
    Element {
        #[doc = include_str!("view_node.doc/component.md")]
        component: ComponentId,
        #[doc = include_str!("view_node.doc/key.md")]
        key: Option<ViewKey>,
        #[doc = include_str!("view_node.doc/props.md")]
        props: Vec<ViewProperty>,
        #[doc = include_str!("view_node.doc/events.md")]
        events: Vec<ViewEventBinding>,
        #[doc = include_str!("view_node.doc/children.md")]
        children: ViewFragment,
    },
    #[doc = include_str!("view_node.doc/Text.md")]
    Text {
        #[doc = include_str!("view_node.doc/key_2.md")]
        key: Option<ViewKey>,
        #[doc = include_str!("view_node.doc/content.md")]
        content: Box<str>,
    },
}

impl ViewNode {
    #[doc = include_str!("view_node.doc/element.md")]
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

    #[doc = include_str!("view_node.doc/slot.md")]
    #[must_use]
    pub fn slot(name: impl Into<Box<str>>) -> Self {
        Self::element(super::SLOT_OUTLET).with_prop(ViewProperty::new(
            super::SLOT,
            super::ViewValue::Text(name.into()),
        ))
    }

    #[doc = include_str!("view_node.doc/virtual_collection.md")]
    #[must_use]
    pub fn virtual_collection(row_extent: f32, template: ViewFragment) -> Self {
        Self::element(super::VIRTUAL_COLLECTION)
            .with_prop(ViewProperty::new(
                super::VIRTUAL_ROW_EXTENT,
                super::ViewValue::Float(f64::from(row_extent)),
            ))
            .with_children(template)
    }

    #[doc = include_str!("view_node.doc/virtual_list.md")]
    #[must_use]
    pub fn virtual_list(list_id: InternedSymbol, count: usize, row_extent: f32) -> Self {
        Self::element(super::VIRTUAL_COLLECTION)
            .with_prop(ViewProperty::new(
                super::VIRTUAL_LIST_ID,
                super::ViewValue::Int(i64::from(list_id.raw())),
            ))
            .with_prop(ViewProperty::new(
                super::VIRTUAL_ITEM_COUNT,
                super::ViewValue::Int(count as i64),
            ))
            .with_prop(ViewProperty::new(
                super::VIRTUAL_ROW_EXTENT,
                super::ViewValue::Float(f64::from(row_extent)),
            ))
    }

    #[doc = include_str!("view_node.doc/with_virtual_scroll_reset.md")]
    #[must_use]
    pub fn with_virtual_scroll_reset(self, token: u64) -> Self {
        #[expect(
            clippy::cast_possible_wrap,
            reason = "the token is opaque; the runtime compares its bit pattern, not its magnitude"
        )]
        let value = super::ViewValue::Int(token as i64);
        self.with_prop(ViewProperty::new(super::VIRTUAL_SCROLL_RESET, value))
    }

    #[doc = include_str!("view_node.doc/text.md")]
    #[must_use]
    pub fn text(content: impl Into<Box<str>>) -> Self {
        Self::Text {
            key: None,
            content: content.into(),
        }
    }

    #[doc = include_str!("view_node.doc/key_3.md")]
    #[must_use]
    pub fn key(&self) -> Option<&ViewKey> {
        match self {
            Self::Element { key, .. } | Self::Text { key, .. } => key.as_ref(),
        }
    }

    #[doc = include_str!("view_node.doc/with_key.md")]
    #[must_use]
    pub fn with_key(mut self, new_key: impl Into<ViewKey>) -> Self {
        let new_key = new_key.into();
        match &mut self {
            Self::Element { key, .. } | Self::Text { key, .. } => *key = Some(new_key),
        }
        self
    }

    #[doc = include_str!("view_node.doc/with_prop.md")]
    #[must_use]
    pub fn with_prop(mut self, prop: ViewProperty) -> Self {
        if let Self::Element { props, .. } = &mut self {
            props.push(prop);
        }
        self
    }

    #[doc = include_str!("view_node.doc/with_event.md")]
    #[must_use]
    pub fn with_event(mut self, event: ViewEventBinding) -> Self {
        if let Self::Element { events, .. } = &mut self {
            events.push(event);
        }
        self
    }

    #[doc = include_str!("view_node.doc/with_children.md")]
    #[must_use]
    pub fn with_children(mut self, new_children: ViewFragment) -> Self {
        if let Self::Element { children, .. } = &mut self {
            *children = new_children;
        }
        self
    }

    #[doc = include_str!("view_node.doc/class.md")]
    #[must_use]
    pub fn class(self, class: impl Into<Box<str>>) -> Self {
        self.with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
    }

    #[doc = include_str!("view_node.doc/style.md")]
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
