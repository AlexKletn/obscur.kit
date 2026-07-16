//! `UiIrViewBridge`: lowers compiled UI IR into a desired-view [`View`]
//! (spec §2.15, §2.11).

use obscur_ui_ir::document::{UiIrDocument, UiIrExpression, UiIrNode, UiIrNodeKind, UiIrValue};
use obscur_ui_ir::name_binding::{self, TagPosition};
use obscur_view::view::{
    View, ViewEventBinding, ViewFragment, ViewKey, ViewNode, ViewProperty, ViewValue,
};

/// Lowers a compiled [`UiIrDocument`] (or a bare [`UiIrNode`] subtree) into the
/// runtime desired [`View`], connecting the frontend IR to the reconciler
/// (spec §2.15).
///
/// # Mapping
///
/// - **Element / Component** → a [`ViewNode::Element`] whose
///   [`ComponentId`](obscur_identity::ComponentId) is resolved through the shared
///   [`name_binding`] registry: a built-in widget/primitive tag binds to its real
///   `COMPONENT_ID`; otherwise the node's **kind** decides — an
///   [`Element`](UiIrNodeKind::Element) binds to the generic `Container`, a
///   [`Component`](UiIrNodeKind::Component) to its own
///   [`user_component_id`](name_binding::user_component_id), which is the id
///   `#[derive(Contract)]` gives the same name and therefore the id a host
///   registers the component under. IR
///   [`properties`](UiIrNode::properties) become [`ViewProperty`]s (the reserved
///   `class`/`id`/`type`/`src`/`disabled`/`checked` attributes map to the
///   `obscur-view` symbols the reconciler/matcher read; every other name is
///   deterministically hashed). IR [`events`](UiIrNode::events) become
///   [`ViewEventBinding`]s (`click`/`input`/`change` bind to the reserved
///   `EVENT_*` symbols the runtime dispatcher resolves) and the node's optional
///   [`key`](UiIrNode::key) becomes a [`ViewKey`].
/// - **Text** → a [`ViewNode::Text`] carrying the literal content (and key, if
///   any).
/// - **Fragment** → its children spliced into the surrounding fragment (a
///   fragment is transparent).
/// - **Conditional / Loop** → *static materialization*: the desired tree is
///   concrete data with no conditional or loop node, so a
///   [`Conditional`](UiIrNodeKind::Conditional) lowers to its guarded body and a
///   [`Loop`](UiIrNodeKind::Loop) lowers to its template body, both spliced into
///   the surrounding fragment. Evaluating a condition's truth or materializing a
///   `<For>` per collection item needs runtime data (the bound values), which
///   the IR does not carry; that dynamic expansion is the runtime's job. The
///   loop's mandatory `key` and `each` binding are preserved on the IR node for
///   that later expansion.
///
/// A property or event value that is a literal lowers to the matching
/// [`ViewValue`]; a dynamic [`Binding`](UiIrExpression::Binding) lowers to its
/// path as placeholder text (its real value needs runtime resolution) and a
/// [`Call`](UiIrExpression::Call) lowers to [`ViewValue::Null`].
#[derive(Debug, Clone, Copy, Default)]
pub struct UiIrViewBridge;

impl UiIrViewBridge {
    /// Lowers a whole [`UiIrDocument`] into a [`View`].
    #[must_use]
    pub fn document_to_view(document: &UiIrDocument) -> View {
        View::from_fragment(ViewFragment::from_nodes(Self::node_to_view_nodes(
            document.root(),
        )))
    }

    /// Lowers one IR `node` into the desired-view node(s) it produces.
    ///
    /// Most kinds produce exactly one [`ViewNode`]; a transparent
    /// [`Fragment`](UiIrNodeKind::Fragment) and the statically materialized
    /// [`Conditional`](UiIrNodeKind::Conditional)/[`Loop`](UiIrNodeKind::Loop)
    /// splice their children, so the return is a `Vec`.
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
            UiIrNodeKind::Fragment | UiIrNodeKind::Conditional | UiIrNodeKind::Loop => node
                .children()
                .iter()
                .flat_map(Self::node_to_view_nodes)
                .collect(),
        }
    }
}

/// Lowers an element/component IR node into a single [`ViewNode::Element`].
///
/// Names bind through the shared [`name_binding`] registry: the tag resolves to
/// a real [`ComponentId`](obscur_identity::ComponentId) (a built-in widget or
/// primitive; else the generic `Container` for an element, or the component's own
/// name-derived id for a component), reserved attributes/events resolve to the
/// `obscur-view` symbols the reconciler/matcher/dispatcher read, and the handler
/// command resolves to a deterministic hashed symbol.
///
/// `position` comes from the caller's [`UiIrNodeKind`] rather than from the tag's
/// spelling: the front ends already decided component-vs-element and recorded it
/// on the node, and re-deriving it here would be a second copy of their rule.
fn lower_element(node: &UiIrNode, position: TagPosition) -> ViewNode {
    let mut view = ViewNode::element(name_binding::component_id_for(node.tag(), position));
    for property in node.properties() {
        view = view.with_prop(ViewProperty::new(
            name_binding::attr_symbol(property.name()),
            lower_value(property.value()),
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

/// Lowers an IR value expression into a concrete [`ViewValue`].
fn lower_value(expression: &UiIrExpression) -> ViewValue {
    match expression {
        UiIrExpression::Literal(value) => lower_literal(value),
        // A dynamic binding has no compile-time value; keep its path visible as
        // placeholder text so the property is still observable in the seam.
        UiIrExpression::Binding(path) => ViewValue::Text(path.clone()),
        UiIrExpression::Call { .. } => ViewValue::Null,
    }
}

/// Lowers an IR literal into the matching [`ViewValue`].
fn lower_literal(value: &UiIrValue) -> ViewValue {
    match value {
        UiIrValue::Null => ViewValue::Null,
        UiIrValue::Bool(flag) => ViewValue::Bool(*flag),
        UiIrValue::Int(number) => ViewValue::Int(*number),
        UiIrValue::Float(number) => ViewValue::Float(*number),
        UiIrValue::Text(text) => ViewValue::Text(text.clone()),
    }
}

/// Lowers an IR key expression into a [`ViewKey`]. Literal ints become index
/// keys; every other shape becomes a name key over its textual form.
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
    use obscur_view::view::{CLASS, EVENT_CLICK, ID, SRC};

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
