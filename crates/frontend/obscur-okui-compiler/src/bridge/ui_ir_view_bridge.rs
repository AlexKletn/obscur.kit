//! `UiIrViewBridge`: lowers compiled UI IR into a desired-view [`View`]
//! (spec §2.15, §2.11).

use obscur_identity::{ComponentId, InternedSymbol};
use obscur_ui_ir::document::{UiIrDocument, UiIrExpression, UiIrNode, UiIrNodeKind, UiIrValue};
use obscur_view::view::{
    CLASS, ID, TYPE, View, ViewEventBinding, ViewFragment, ViewKey, ViewNode, ViewProperty,
    ViewValue,
};

/// Lowers a compiled [`UiIrDocument`] (or a bare [`UiIrNode`] subtree) into the
/// runtime desired [`View`], connecting the frontend IR to the reconciler
/// (spec §2.15).
///
/// # Mapping
///
/// - **Element / Component** → a [`ViewNode::Element`] whose
///   [`ComponentId`] is derived deterministically from the tag/component name
///   (a stable placeholder until a real name interner is threaded through — the
///   numeric handle carries no external meaning). IR
///   [`properties`](UiIrNode::properties) become [`ViewProperty`]s (the
///   well-known `class`/`id`/`type` names map to the reserved
///   [`CLASS`]/[`ID`]/[`TYPE`] symbols so the reconciler's CSS attributes flow
///   through; every other name is deterministically interned). IR
///   [`events`](UiIrNode::events) become [`ViewEventBinding`]s and the node's
///   optional [`key`](UiIrNode::key) becomes a [`ViewKey`].
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
            UiIrNodeKind::Element | UiIrNodeKind::Component => vec![lower_element(node)],
            UiIrNodeKind::Fragment | UiIrNodeKind::Conditional | UiIrNodeKind::Loop => {
                node.children().iter().flat_map(Self::node_to_view_nodes).collect()
            }
        }
    }
}

/// Lowers an element/component IR node into a single [`ViewNode::Element`].
fn lower_element(node: &UiIrNode) -> ViewNode {
    let mut view = ViewNode::element(ComponentId::from_raw(hash_name_64(node.tag())));
    for property in node.properties() {
        view = view.with_prop(ViewProperty::new(
            intern_name(property.name()),
            lower_value(property.value()),
        ));
    }
    for event in node.events() {
        view = view.with_event(ViewEventBinding::new(
            intern_name(event.event()),
            intern_name(event.handler()),
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
        UiIrExpression::Literal(UiIrValue::Float(number)) => ViewKey::from(number.to_string().as_str()),
        UiIrExpression::Literal(UiIrValue::Null) => ViewKey::from("null"),
        UiIrExpression::Call { name, .. } => ViewKey::from(name.as_ref()),
    }
}

/// Interns a property/event name into an [`InternedSymbol`].
///
/// The CSS-relevant `class`/`id`/`type` names map to the reserved
/// [`CLASS`]/[`ID`]/[`TYPE`] symbols so the reconciler's style attributes flow
/// through; any other name is hashed into a high symbol namespace (top bit set)
/// that never collides with the reserved low range. This is a deterministic
/// placeholder until a shared name interner is threaded through the pipeline.
fn intern_name(name: &str) -> InternedSymbol {
    match name {
        "class" => CLASS,
        "id" => ID,
        "type" => TYPE,
        other => InternedSymbol::from_raw(hash_name_32(other) | 0x8000_0000),
    }
}

/// Deterministic FNV-1a 64-bit hash of a name into a component handle.
fn hash_name_64(name: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in name.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Deterministic FNV-1a 32-bit hash of a name into a symbol handle.
fn hash_name_32(name: &str) -> u32 {
    let mut hash: u32 = 0x811c_9dc5;
    for byte in name.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_ui_ir::document::{UiIrEvent, UiIrProperty};

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
        assert_eq!(*component, ComponentId::from_raw(hash_name_64("Button")));
        assert_eq!(key.as_ref(), Some(&ViewKey::from("k")));

        // The `id` property maps onto the reserved ID symbol as text.
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].name(), ID);
        assert_eq!(props[0].value(), &ViewValue::Text("save".into()));

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event(), intern_name("click"));
        assert_eq!(events[0].command(), intern_name("submit"));

        // The text child is a real text node.
        assert_eq!(children.len(), 1);
        assert_eq!(children.nodes()[0], ViewNode::text("Save"));
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
        assert!(view.root().nodes().iter().all(|n| matches!(n, ViewNode::Element { .. })));
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
