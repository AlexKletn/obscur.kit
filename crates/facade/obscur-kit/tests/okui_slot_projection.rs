//! End-to-end: an authored `<h1 slot="header"/>` **reaches the component's
//! header outlet**, and reaches no other one (spec §2.12, §2.13).
//!
//! # What this closes
//!
//! §2.12's slots were declared and enforced but routed nowhere. A component
//! contract could name a slot, both front ends checked an author's `slot="…"`
//! against that declaration — and then `<Card><h1 slot="header"/></Card>`
//! rendered Card and **dropped the h1**, because a component's output named no
//! position for it to go to. `okui_slots.rs` said so out loud: "What does not
//! happen is *projection*."
//!
//! The missing half was the outlet, not the signature.
//! [`ViewNode::slot("header")`](obscur_kit::view::view::ViewNode::slot) is a
//! component's way of saying *where* its `header` content goes; the reconciler
//! keeps *what* goes there (`RetainedNode::slot_content`); and
//! `UiRuntime::render_components` joins them with `project_slots` before the
//! component's output is committed.
//!
//! # Why `Component::render` did not change
//!
//! Because a slot is a **declaration**, and handing a component its children
//! would leave the declaration inert. §2.12 says a component *declares* named
//! slots and §2.13 exports `slots` as a field of its contract — separate from
//! `props`. A `render(ctx, props, children)` signature answers a different
//! question: it gives a component the *content* but not the *position*, so every
//! component would re-implement grouping-by-slot-name in Rust and the contract's
//! declared slots would still route nothing. That is this repo's signature defect
//! — a declaration nobody acts on — bought at the price of an authoring-trait
//! change.
//!
//! Children-in-`props` loses on §2.13 (which lists props and slots separately)
//! and on the type: a prop is a `ViewValue`, a scalar/text union that the CSS
//! matcher and `StyleAttributes::extract` walk, and a subtree does not belong in
//! it. Children-in-`ctx` loses on §3.1, which enumerates how state enters a
//! component (typed props, read-only sources, local state) and does not include
//! the scope handle — and it would let a component that declares no slot read
//! content anyway, making the contract's `slots` field advisory.
//!
//! # What is NOT claimed
//!
//! * **No shipped component declares a slot**, because no shipped *widget is a
//!   `Component`* at all — every `impl Component` in this repo is a test's. Slots
//!   are toolkit API for a host's own components, exercised here exactly as a host
//!   would. `Dialog` still takes one anonymous child list and is a view-builder,
//!   not a component; giving it slots means making it a component first.
//! * **A `.okui`/`okui!` document cannot write an outlet.** `<slot name="header"/>`
//!   is not a tag either front end knows: `name_binding::attr_symbol` binds an
//!   attribute by its own spelling with no idea which tag it sits on, so `name`
//!   cannot mean the slot's name on one tag and a hashed prop everywhere else. A
//!   component's body is hand-written Rust today regardless — §2.12's `Template`
//!   is still a node the parser never emits — so the outlet is authored where the
//!   body is. `an_outlet_has_no_authored_spelling_yet` pins it.
//! * **An outlet outside a component's output is an inert box.** Nothing projects
//!   a view that is not some component's render, so a stray outlet there is not
//!   substituted away. `a_stray_outlet_outside_a_component_is_inert` pins it.
#![forbid(unsafe_code)]

use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::okui_compiler::bridge::UiIrViewBridge;
use obscur_kit::okui_compiler::compile::Compiler;
use obscur_kit::okui_syntax::ast::Document;
use obscur_kit::okui_syntax::parser::Parser;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::ui_macros::{Contract, okui};
use obscur_kit::view::component::{
    Component, ComponentContext, ComponentContract, ComponentFactory, ComponentInstance,
    ComponentRegistration, ComponentVersion, ErasedComponent, InstanceId,
};
use obscur_kit::view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// `Card` declares two §2.12 slots and renders **two regions**, each holding the
/// outlet of one of them.
///
/// Two, not one, on purpose: an assertion that the `h1` landed *somewhere* under
/// Card passes for an implementation that ignores the name and dumps every child
/// into the first outlet it finds. The `body` region is what makes "reached
/// `header`" mean more than "reached Card".
#[derive(Contract)]
#[contract(name = "Card")]
#[contract(slot(name = "header", cardinality = one))]
#[contract(slot(name = "body", cardinality = many))]
struct CardProps {
    /// A required prop, so props and slots are exercised together.
    title: String,
}

struct Card;

impl Component for Card {
    /// `<region name="header"><slot name="header"/></region>` twice over — the
    /// component says *where*, and nothing about *what*.
    fn render(&mut self, _ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::text(format!("card:{}", props.title)),
            region("header"),
            region("body"),
        ]))
    }

    type Props = CardProps;
}

/// The reserved `class` symbol, so a region is identifiable in the retained tree.
fn class() -> obscur_kit::identity::InternedSymbol {
    obscur_kit::view::view::CLASS
}

/// A box classed `region-<slot>` wrapping the outlet for `slot`.
fn region(slot: &str) -> ViewNode {
    ViewNode::element(container())
        .with_prop(ViewProperty::new(
            class(),
            ViewValue::Text(format!("region-{slot}").into()),
        ))
        .with_children(ViewFragment::from_nodes(vec![ViewNode::slot(slot)]))
}

fn container() -> ComponentId {
    obscur_kit::ui_ir::name_binding::CONTAINER
}

#[derive(Debug)]
struct CardFactory;

impl ComponentFactory for CardFactory {
    fn component_id(&self) -> ComponentId {
        CardProps::contract().id
    }
    fn contract(&self) -> ComponentContract {
        CardProps::contract()
    }
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, CardProps::contract().id)
    }
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(Card)
    }
}

fn runtime_with_card() -> UiRuntime {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));
    runtime
        .register_component(ComponentRegistration::new(
            Box::new(CardFactory),
            ComponentVersion::INITIAL,
        ))
        .expect("Card registers");
    runtime
}

/// Every text leaf beneath the retained node classed `class_name`, in tree order.
///
/// Reading through the *class* rather than through a child index is what keeps
/// the assertion honest: an index would still be satisfied if the two regions
/// swapped places.
fn texts_in_region(runtime: &UiRuntime, class_name: &str) -> Vec<String> {
    let Some(root) = runtime.tree().root() else {
        return Vec::new();
    };
    let mut found = Vec::new();
    let Some(region) = find_by_class(runtime, root, class_name) else {
        return found;
    };
    collect_text(runtime, region, &mut found);
    found
}

fn find_by_class(
    runtime: &UiRuntime,
    node: obscur_kit::identity::NodeId,
    class_name: &str,
) -> Option<obscur_kit::identity::NodeId> {
    let retained = runtime.tree().get(node)?;
    if retained
        .attributes()
        .classes()
        .iter()
        .any(|c| c.as_ref() == class_name)
    {
        return Some(node);
    }
    retained
        .children()
        .iter()
        .find_map(|&child| find_by_class(runtime, child, class_name))
}

fn collect_text(runtime: &UiRuntime, node: obscur_kit::identity::NodeId, out: &mut Vec<String>) {
    let Some(retained) = runtime.tree().get(node) else {
        return;
    };
    if let obscur_kit::tree::node::NodeKind::Text { content } = retained.kind() {
        out.push(content.to_string());
    }
    for &child in retained.children() {
        collect_text(runtime, child, out);
    }
}

/// Parses and compiles `source` through the real `.okui` pipeline.
fn compile(source: &str) -> obscur_kit::ui_ir::document::UiIrDocument {
    let parsed = Parser::new().parse(source);
    let document = Document::cast(parsed.tree().root().clone()).expect("a document");
    let result = Compiler::new("card.okui").compile(&document);
    assert!(
        !result.has_errors(),
        "diagnostics: {:?}",
        result.diagnostics()
    );
    result.document().expect("a lowered document").clone()
}

// --- THE test. -----------------------------------------------------------

#[test]
fn authored_content_reaches_the_outlet_that_names_its_slot_and_not_the_other() {
    // The finding, closed. Before the outlet existed, both regions were empty and
    // the `h1`/`p` were dropped on the floor by a component that had nowhere to
    // put them.
    let mut runtime = runtime_with_card();
    let view: View = okui! {
        <Card title="Hi">
            <h1 slot="header">"Title"</h1>
            <p slot="body">"First"</p>
            <p slot="body">"Second"</p>
        </Card>
    };
    runtime.render_now(&view, &[]);

    assert_eq!(
        texts_in_region(&runtime, "region-header"),
        ["Title"],
        "the h1 assigned to `header` reached the header outlet"
    );
    assert_eq!(
        texts_in_region(&runtime, "region-body"),
        ["First", "Second"],
        "a `Many` slot takes every child that names it, in authored order"
    );
    // The disjoint half, and the reason there are two regions: routing that
    // ignored the name would put all three under `header`.
    assert!(
        !texts_in_region(&runtime, "region-header").contains(&"First".to_string()),
        "content assigned to `body` must NOT reach the header outlet"
    );
    assert!(
        !texts_in_region(&runtime, "region-body").contains(&"Title".to_string()),
        "content assigned to `header` must NOT reach the body outlet"
    );
}

#[test]
fn the_file_front_end_projects_the_same_document() {
    // Both front ends gain a change together — the house rule the
    // `<VirtualFor>`-was-`<For>` drift was written to enforce. Neither front end
    // needed new lowering here (both already bind `slot="…"` through the one
    // `name_binding::attr_symbol`), which is exactly what this asserts.
    let ir =
        compile(r#"<Card title="Hi"><h1 slot="header">Title</h1><p slot="body">First</p></Card>"#);
    let view = UiIrViewBridge::document_to_view(&ir);

    let mut runtime = runtime_with_card();
    runtime.render_now(&view, &[]);

    assert_eq!(texts_in_region(&runtime, "region-header"), ["Title"]);
    assert_eq!(texts_in_region(&runtime, "region-body"), ["First"]);
}

#[test]
fn projection_happens_on_the_components_opening_frame() {
    // Not a latency bug hidden by a second render. The content is captured from
    // the authored data, so it is on the host node *before* the mount that the
    // same plan drives — one `render_now`, content on screen.
    let mut runtime = runtime_with_card();
    let view: View = okui! { <Card title="Hi"><h1 slot="header">"Title"</h1></Card> };
    runtime.render_now(&view, &[]);
    assert_eq!(texts_in_region(&runtime, "region-header"), ["Title"]);
}

#[test]
fn changed_slot_content_re_renders_the_host_and_re_projects() {
    // The half a props-only diff would miss: `title` is identical across these
    // two renders, so nothing but the slot content changed. Without the content
    // in the node's change check there is no `Update`, the component is never
    // marked stale, and the header keeps showing "Before".
    let mut runtime = runtime_with_card();
    runtime.render_now(
        &okui! { <Card title="Hi"><h1 slot="header">"Before"</h1></Card> },
        &[],
    );
    assert_eq!(texts_in_region(&runtime, "region-header"), ["Before"]);

    runtime.render_now(
        &okui! { <Card title="Hi"><h1 slot="header">"After"</h1></Card> },
        &[],
    );
    assert_eq!(
        texts_in_region(&runtime, "region-header"),
        ["After"],
        "content that changed while props did not must still re-project"
    );
}

#[test]
fn an_unfilled_outlet_leaves_no_node_behind() {
    // An outlet is substituted away, not retained: an unfilled slot must not
    // become an empty box that lays out and paints.
    let mut runtime = runtime_with_card();
    runtime.render_now(&okui! { <Card title="Hi"/> }, &[]);

    let region = find_by_class(
        &runtime,
        runtime.tree().root().expect("a root"),
        "region-header",
    )
    .expect("the region itself is the component's own markup and stays");
    assert!(
        runtime
            .tree()
            .get(region)
            .expect("live")
            .children()
            .is_empty(),
        "the outlet is gone, not standing empty"
    );
}

#[test]
fn a_document_authoring_no_slot_stores_no_slot_content() {
    // The gate that makes retaining authored children affordable. Every element's
    // children are its whole subtree, so keeping them unconditionally would clone
    // the desired tree once per level of depth. A document with no `slot=` pays a
    // scan and stores nothing.
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(2), Size::new(200.0, 200.0));
    runtime.render_now(&okui! { <div><span>"x"</span></div> }, &[]);

    let root = runtime.tree().root().expect("a root");
    let div = runtime.tree().get(root).expect("live").children()[0];
    assert!(
        runtime
            .tree()
            .get(div)
            .expect("live")
            .slot_content()
            .is_empty(),
        "no authored slot, no retained content"
    );
}

// --- The residue, pinned rather than hidden. -----------------------------

#[test]
fn an_outlet_has_no_authored_spelling_yet() {
    // `<slot name="header"/>` is not a tag either front end knows: it binds like
    // any other unknown element tag — a generic box — and `name` hashes to an
    // ordinary prop. A component's body is hand-written Rust regardless (§2.12's
    // `Template` is a node the parser never emits), so this is a gap in the
    // authoring surface for component bodies, not in the routing.
    let view: View = okui! { <slot name="header"/> };
    let ViewNode::Element { component, .. } = &view.root().nodes()[0] else {
        panic!("a tag lowers to an element");
    };
    assert_eq!(
        *component,
        container(),
        "`<slot>` is still just an unknown element tag — the outlet is Rust-only"
    );
    assert_ne!(*component, obscur_kit::view::view::SLOT_OUTLET);
}

#[test]
fn a_stray_outlet_outside_a_component_is_inert() {
    // Projection runs over a component's render output. A view that is nobody's
    // render output is never projected, so an outlet in one is not substituted
    // away — it lands as an ordinary empty element. Not fail-closed, and said out
    // loud rather than left for someone to discover.
    let mut runtime = runtime_with_card();
    runtime.render_now(&View::single(ViewNode::slot("header")), &[]);

    let root = runtime.tree().root().expect("a root");
    assert_eq!(
        runtime.tree().get(root).expect("live").children().len(),
        1,
        "the stray outlet became a real node"
    );
}
