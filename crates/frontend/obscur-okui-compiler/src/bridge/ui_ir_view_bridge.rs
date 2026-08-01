#![doc = include_str!("ui_ir_view_bridge.doc/_module.md")]

use obscur_ui_ir::document::{
    UiIrDocument, UiIrExpression, UiIrNode, UiIrNodeKind, UiIrProperty, UiIrValue,
};
use obscur_ui_ir::name_binding::{self, TagPosition};
use obscur_view::view::{
    View, ViewEventBinding, ViewFragment, ViewKey, ViewNode, ViewProperty, ViewValue,
};

#[doc = include_str!("ui_ir_view_bridge.doc/UiIrViewBridge.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct UiIrViewBridge;

impl UiIrViewBridge {
    #[doc = include_str!("ui_ir_view_bridge.doc/document_to_view.md")]
    #[must_use]
    pub fn document_to_view(document: &UiIrDocument) -> View {
        View::from_fragment(ViewFragment::from_nodes(Self::node_to_view_nodes(
            document.root(),
        )))
    }

    #[doc = include_str!("ui_ir_view_bridge.doc/node_to_view_nodes.md")]
    #[must_use]
    pub fn node_to_view_nodes(node: &UiIrNode) -> Vec<ViewNode> {
        match node.kind() {
            UiIrNodeKind::Text => {
                let mut view = ViewNode::text(node.content());
                if let Some(key) = node.key() {
                    view = view.with_key(lower_key(key));
                }
                vec![view]
            }
            UiIrNodeKind::Element => vec![lower_element(node, TagPosition::Element)],
            UiIrNodeKind::Component => vec![lower_element(node, TagPosition::Component)],
            // A `<VirtualFor>` is *not* a `<For>`: it lowers to a single
            // virtual-collection marker carrying its row template, which the
            // runtime realizes into a viewport-bounded window (spec §4.9) rather
            // than materializing one node per item.
            UiIrNodeKind::Loop if node.tag() == "VirtualFor" => {
                vec![lower_virtual_collection(node)]
            }
            UiIrNodeKind::Fragment | UiIrNodeKind::Conditional | UiIrNodeKind::Loop => node
                .children()
                .iter()
                .flat_map(Self::node_to_view_nodes)
                .collect(),
        }
    }
}

#[doc = include_str!("ui_ir_view_bridge.doc/lower_element.md")]
fn lower_element(node: &UiIrNode, position: TagPosition) -> ViewNode {
    let mut view = ViewNode::element(name_binding::component_id_for(node.tag(), position));
    for property in node.properties() {
        view = view.with_prop(ViewProperty::new(
            // Through the *tag-aware* binding, so `<slot name="header"/>` lowers to
            // the §2.12 outlet rather than to a box with a hashed `name` prop. The
            // tag was always here: this loop has `node.tag()` in scope and is
            // already threading the node's position from it.
            name_binding::attr_symbol_on(node.tag(), property.name()),
            lower_value(property.value()),
        ));
    }
    // The tag itself, as the reserved TYPE property — the element's local name, so
    // an authored `<button>` is what a `button { … }` type selector matches (spec
    // §4.13). Appended after the authored properties so each of those keeps its
    // authored position, and skipped when the author bound `type` themselves; both
    // rules live in `local_name_for`, which the code generator calls with the same
    // arguments to emit the same property.
    if let Some(local_name) = name_binding::local_name_for(
        node.tag(),
        position,
        node.properties().iter().map(UiIrProperty::name),
    ) {
        view = view.with_prop(ViewProperty::new(
            name_binding::TYPE,
            ViewValue::Text(local_name.into()),
        ));
    }
    for event in node.events() {
        view = view.with_event(ViewEventBinding::new(
            name_binding::event_symbol(event.event()),
            name_binding::fallback_symbol(event.handler()),
        ));
    }
    let children: ViewFragment = node
        .children()
        .iter()
        .flat_map(UiIrViewBridge::node_to_view_nodes)
        .collect();
    view = view.with_children(children);
    if let Some(key) = node.key() {
        view = view.with_key(lower_key(key));
    }
    view
}

#[doc = include_str!("ui_ir_view_bridge.doc/lower_virtual_collection.md")]
fn lower_virtual_collection(node: &UiIrNode) -> ViewNode {
    let template: ViewFragment = node
        .children()
        .iter()
        .flat_map(UiIrViewBridge::node_to_view_nodes)
        .collect();
    ViewNode::virtual_collection(obscur_view::view::DEFAULT_VIRTUAL_ROW_EXTENT, template)
}

#[doc = include_str!("ui_ir_view_bridge.doc/lower_value.md")]
fn lower_value(expression: &UiIrExpression) -> ViewValue {
    match expression {
        UiIrExpression::Literal(value) => lower_literal(value),
        // A dynamic binding has no compile-time value; keep its path visible as
        // placeholder text so the property is still observable in the seam.
        UiIrExpression::Binding(path) => ViewValue::Text(path.clone()),
        UiIrExpression::Call { .. } => ViewValue::Null,
    }
}

#[doc = include_str!("ui_ir_view_bridge.doc/lower_literal.md")]
fn lower_literal(value: &UiIrValue) -> ViewValue {
    match value {
        UiIrValue::Null => ViewValue::Null,
        UiIrValue::Bool(flag) => ViewValue::Bool(*flag),
        UiIrValue::Int(number) => ViewValue::Int(*number),
        UiIrValue::Float(number) => ViewValue::Float(*number),
        UiIrValue::Text(text) => ViewValue::Text(text.clone()),
    }
}

#[doc = include_str!("ui_ir_view_bridge.doc/lower_key.md")]
fn lower_key(expression: &UiIrExpression) -> ViewKey {
    match expression {
        UiIrExpression::Literal(UiIrValue::Int(number)) => ViewKey::from(number.cast_unsigned()),
        UiIrExpression::Literal(UiIrValue::Text(text)) => ViewKey::from(text.as_ref()),
        UiIrExpression::Binding(path) => ViewKey::from(path.as_ref()),
        UiIrExpression::Literal(UiIrValue::Bool(flag)) => {
            ViewKey::from(if *flag { "true" } else { "false" })
        }
        UiIrExpression::Literal(UiIrValue::Float(number)) => {
            ViewKey::from(number.to_string().as_str())
        }
        UiIrExpression::Literal(UiIrValue::Null) => ViewKey::from("null"),
        UiIrExpression::Call { name, .. } => ViewKey::from(name.as_ref()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;
    use obscur_ui_ir::document::{UiIrEvent, UiIrProperty};
    use obscur_view::view::{CLASS, EVENT_CLICK, ID, SRC, TYPE};

    #[test]
    fn element_text_event_and_key_map_to_view_nodes() {
        // <Button id="save" on:click=submit key="k">Save</Button>
        let root = UiIrNode::component(1, "Button")
            .with_key(UiIrExpression::text("k"))
            .with_property(UiIrProperty::text("id", "save"))
            .with_event(UiIrEvent::new("click", "submit"))
            .with_child(UiIrNode::text(2, "Save"));
        let doc = UiIrDocument::new(root);

        let view = UiIrViewBridge::document_to_view(&doc);
        let nodes = view.root().nodes();
        assert_eq!(nodes.len(), 1);

        let ViewNode::Element {
            component,
            key,
            props,
            events,
            children,
        } = &nodes[0]
        else {
            panic!("expected an element");
        };
        // The `Button` tag binds to the real widget COMPONENT_ID, not a hash.
        assert_eq!(*component, ComponentId::from_raw(0x00D0_0001));
        assert_eq!(key.as_ref(), Some(&ViewKey::from("k")));

        // The `id` property maps onto the reserved ID symbol as text.
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].name(), ID);
        assert_eq!(props[0].value(), &ViewValue::Text("save".into()));

        // The `click` event binds to the reserved EVENT_CLICK symbol the runtime
        // dispatcher resolves, so a real click reaches its command.
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event(), EVENT_CLICK);
        assert_eq!(events[0].command(), name_binding::fallback_symbol("submit"));

        // The text child is a real text node.
        assert_eq!(children.len(), 1);
        assert_eq!(children.nodes()[0], ViewNode::text("Save"));
    }

    #[test]
    fn img_src_reaches_the_reserved_src_symbol() {
        // <img src="logo.png"/> — the image element binds to the real Image
        // primitive and its `src` reaches the reserved SRC symbol the runtime
        // resolves to decoded pixels (the old hash bridge dropped both).
        let root = UiIrNode::element(1, "img").with_property(UiIrProperty::text("src", "logo.png"));
        let view = UiIrViewBridge::document_to_view(&UiIrDocument::new(root));
        let ViewNode::Element {
            component, props, ..
        } = &view.root().nodes()[0]
        else {
            panic!("element");
        };
        assert_eq!(*component, ComponentId::from_raw(0x00C0_0003));
        assert_eq!(props[0].name(), SRC);
        assert_eq!(props[0].value(), &ViewValue::Text("logo.png".into()));
    }

    #[test]
    fn generic_div_binds_to_the_container_primitive() {
        let root = UiIrNode::element(1, "div").with_property(UiIrProperty::text("class", "row"));
        let view = UiIrViewBridge::document_to_view(&UiIrDocument::new(root));
        let ViewNode::Element { component, .. } = &view.root().nodes()[0] else {
            panic!("element");
        };
        assert_eq!(*component, ComponentId::from_raw(0x00C0_0001));
    }

    #[test]
    fn conditional_materializes_its_body_children() {
        // <If cond={open}><A/><B/></If> -> A, B spliced (the If is transparent).
        let root = UiIrNode::conditional(1, "If")
            .with_property(UiIrProperty::binding("cond", "open"))
            .with_child(UiIrNode::component(2, "A"))
            .with_child(UiIrNode::component(3, "B"));
        let doc = UiIrDocument::new(root);

        let view = UiIrViewBridge::document_to_view(&doc);
        assert_eq!(view.root().len(), 2);
        assert!(
            view.root()
                .nodes()
                .iter()
                .all(|n| matches!(n, ViewNode::Element { .. }))
        );
    }

    #[test]
    fn loop_materializes_its_template_body() {
        // <For each={items} key={id}><Row/></For> -> the Row template, spliced.
        let root = UiIrNode::loop_node(1, "For")
            .with_key(UiIrExpression::binding("id"))
            .with_property(UiIrProperty::binding("each", "items"))
            .with_child(UiIrNode::component(2, "Row"));
        let doc = UiIrDocument::new(root);

        let view = UiIrViewBridge::document_to_view(&doc);
        assert_eq!(view.root().len(), 1);
        assert!(matches!(view.root().nodes()[0], ViewNode::Element { .. }));
    }

    #[test]
    fn fragment_root_splices_its_children() {
        let root = UiIrNode::fragment(1)
            .with_child(UiIrNode::element(2, "a"))
            .with_child(UiIrNode::element(3, "b"));
        let doc = UiIrDocument::new(root);
        assert_eq!(UiIrViewBridge::document_to_view(&doc).root().len(), 2);
    }

    /// The value of `symbol` on the single root element of the view `node` lowers
    /// to, or `None` when the property is absent.
    fn root_prop(node: UiIrNode, symbol: obscur_identity::InternedSymbol) -> Option<ViewValue> {
        let view = UiIrViewBridge::document_to_view(&UiIrDocument::new(node));
        let ViewNode::Element { props, .. } = &view.root().nodes()[0] else {
            panic!("element");
        };
        props
            .iter()
            .find(|prop| prop.name() == symbol)
            .map(|prop| prop.value().clone())
    }

    #[test]
    fn an_authored_button_carries_button_as_its_local_name() {
        // The two halves of "a base element, as in HTML": the shipped widget's id,
        // and the local name a `button { … }` type selector matches (§4.13). The
        // node used to have neither — a `<button>` was an anonymous `div`.
        let view =
            UiIrViewBridge::document_to_view(&UiIrDocument::new(UiIrNode::element(1, "button")));
        let ViewNode::Element {
            component, props, ..
        } = &view.root().nodes()[0]
        else {
            panic!("element");
        };
        assert_eq!(*component, ComponentId::from_raw(0x00D0_0001));
        let attributes = obscur_view::view::StyleAttributes::extract(props);
        assert_eq!(attributes.local_name(), Some("button"));
    }

    #[test]
    fn every_element_tag_reaches_the_tree_with_its_name() {
        // Not a button feature: the whole element vocabulary was anonymous.
        for tag in ["div", "span", "td", "img"] {
            assert_eq!(
                root_prop(UiIrNode::element(1, tag), TYPE),
                Some(ViewValue::Text(tag.into())),
                "{tag}"
            );
        }
    }

    #[test]
    fn an_authored_type_wins_over_the_implicit_local_name() {
        // `<button type="submit">` is HTML's own spelling, and TYPE is the one symbol
        // both channels share: the author's value is the only one on the node.
        let node =
            UiIrNode::element(1, "button").with_property(UiIrProperty::text("type", "submit"));
        let view = UiIrViewBridge::document_to_view(&UiIrDocument::new(node));
        let ViewNode::Element { props, .. } = &view.root().nodes()[0] else {
            panic!("element");
        };
        let types: Vec<&ViewValue> = props
            .iter()
            .filter(|prop| prop.name() == TYPE)
            .map(ViewProperty::value)
            .collect();
        assert_eq!(types, vec![&ViewValue::Text("submit".into())]);
    }

    #[test]
    fn a_component_tag_gets_no_local_name() {
        // A `<Card/>` node is a seam, not a box an author styles by type; the
        // elements its render produces carry their own names.
        assert_eq!(root_prop(UiIrNode::component(1, "Card"), TYPE), None);
        assert_eq!(root_prop(UiIrNode::component(1, "Button"), TYPE), None);
    }

    #[test]
    fn class_property_maps_to_reserved_class_symbol() {
        let root = UiIrNode::element(1, "div").with_property(UiIrProperty::text("class", "row"));
        let view = UiIrViewBridge::document_to_view(&UiIrDocument::new(root));
        let ViewNode::Element { props, .. } = &view.root().nodes()[0] else {
            panic!("element");
        };
        assert_eq!(props[0].name(), CLASS);
    }
}
