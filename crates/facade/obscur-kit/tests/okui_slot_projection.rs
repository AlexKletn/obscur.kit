#![doc = include_str!("okui_slot_projection.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;

use obscur_kit::diagnostics::{Diagnostic, DiagnosticSeverity, DiagnosticSink};
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, FrameId, RuntimeId};
use obscur_kit::okui_compiler::bridge::UiIrViewBridge;
use obscur_kit::okui_compiler::compile::Compiler;
use obscur_kit::okui_syntax::ast::Document;
use obscur_kit::okui_syntax::parser::Parser;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::frame::{FrameContext, FramePipeline, FrameReason};
use obscur_kit::runtime::orchestration::StyleInputs;
use obscur_kit::scene::scene::Scene;
use obscur_kit::tree::tree::RetainedTree;
use obscur_kit::ui_macros::{Contract, okui};
use obscur_kit::view::component::{
    Component, ComponentContext, ComponentContract, ComponentFactory, ComponentInstance,
    ComponentRegistration, ComponentVersion, ErasedComponent, InstanceId,
};
use obscur_kit::view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("okui_slot_projection.doc/CardProps.md")]
#[derive(Contract)]
#[contract(name = "Card")]
#[contract(slot(name = "header", cardinality = one))]
#[contract(slot(name = "body", cardinality = many))]
struct CardProps {
    #[doc = include_str!("okui_slot_projection.doc/title.md")]
    title: String,
}

struct Card;

impl Component for Card {
    #[doc = include_str!("okui_slot_projection.doc/render.md")]
    fn render(&mut self, _ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::text(format!("card:{}", props.title)),
            region("header"),
            region("body"),
        ]))
    }

    type Props = CardProps;
}

#[doc = include_str!("okui_slot_projection.doc/class.md")]
fn class() -> obscur_kit::identity::InternedSymbol {
    obscur_kit::view::view::CLASS
}

#[doc = include_str!("okui_slot_projection.doc/region.md")]
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

#[doc = include_str!("okui_slot_projection.doc/AuthoredCardProps.md")]
#[derive(Contract)]
#[contract(name = "AuthoredCard")]
#[contract(slot(name = "header", cardinality = one))]
#[contract(slot(name = "body", cardinality = many))]
struct AuthoredCardProps {
    #[doc = include_str!("okui_slot_projection.doc/title_2.md")]
    title: String,
}

struct AuthoredCard;

impl Component for AuthoredCard {
    #[doc = include_str!("okui_slot_projection.doc/render_2.md")]
    fn render(&mut self, _ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
        let title = &props.title;
        okui! {
            <div>
                {title}
                <div class="region-header"><slot name="header"/></div>
                <div class="region-body"><slot name="body"/></div>
            </div>
        }
    }

    type Props = AuthoredCardProps;
}

#[derive(Debug)]
struct AuthoredCardFactory;

impl ComponentFactory for AuthoredCardFactory {
    fn component_id(&self) -> ComponentId {
        AuthoredCardProps::contract().id
    }
    fn contract(&self) -> ComponentContract {
        AuthoredCardProps::contract()
    }
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, AuthoredCardProps::contract().id)
    }
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(AuthoredCard)
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

#[doc = include_str!("okui_slot_projection.doc/texts_in_region.md")]
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
        .child_nodes()
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
    for &child in retained.child_nodes() {
        collect_text(runtime, child, out);
    }
}

#[doc = include_str!("okui_slot_projection.doc/compile.md")]
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
            .child_nodes()
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
    let div = runtime.tree().get(root).expect("live").child_nodes()[0];
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

// --- The authored spelling. ----------------------------------------------

#[test]
fn an_authored_slot_tag_is_the_outlet_on_the_macro_front_end() {
    // The residue this file used to pin, closed. `<slot name="header"/>` was an
    // unknown element tag — a generic box with a hashed `name` prop, identical to
    // a `<div>`. It is now the same node `ViewNode::slot("header")` builds.
    let view: View = okui! { <slot name="header"/> };
    assert_eq!(view.root().nodes(), [ViewNode::slot("header")]);
    let ViewNode::Element { component, .. } = &view.root().nodes()[0] else {
        panic!("a tag lowers to an element");
    };
    assert_eq!(*component, obscur_kit::view::view::SLOT_OUTLET);
    assert_ne!(*component, container(), "an outlet is not a box");
}

#[test]
fn an_authored_slot_tag_is_the_outlet_on_the_file_front_end_too() {
    // Both front ends gain it together — the house rule the
    // `<VirtualFor>`-was-`<For>` drift was written to enforce. They gain it from
    // the same two `name_binding` rules, which is *why* neither could have got it
    // alone: the file path's `UiIrViewBridge` and the macro's `ViewCodegen` are
    // the only two callers of `attr_symbol_on`.
    let ir = compile(r#"<slot name="header"/>"#);
    let view = UiIrViewBridge::document_to_view(&ir);
    assert_eq!(view.root().nodes(), [ViewNode::slot("header")]);
}

#[test]
fn a_component_can_author_its_own_render_with_the_slot_tag() {
    // What the spelling is *for*, driven end to end. `AuthoredCard::render`
    // returns the `okui!` expansion verbatim — no hand-built `ViewNode::slot`
    // anywhere — and the authored `<h1 slot="header"/>` reaches the header region
    // and not the body one.
    //
    // This is the whole §2.12 loop closed on one authoring surface: a component
    // declares its slots in its contract, spells the outlets in `okui!`, and a
    // caller assigns to them in `okui!`.
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(3), Size::new(200.0, 200.0));
    runtime
        .register_component(ComponentRegistration::new(
            Box::new(AuthoredCardFactory),
            ComponentVersion::INITIAL,
        ))
        .expect("AuthoredCard registers");
    runtime.render_now(
        &okui! {
            <AuthoredCard title="Hi">
                <h1 slot="header">"Title"</h1>
                <p slot="body">"Body"</p>
            </AuthoredCard>
        },
        &[],
    );

    assert_eq!(texts_in_region(&runtime, "region-header"), ["Title"]);
    assert_eq!(texts_in_region(&runtime, "region-body"), ["Body"]);
    assert!(
        !texts_in_region(&runtime, "region-header").contains(&"Body".to_string()),
        "an authored outlet routes by name, like a hand-built one"
    );
}

// --- Fail-closed: a stray outlet. ----------------------------------------

#[doc = include_str!("okui_slot_projection.doc/Collected.md")]
#[derive(Debug, Default, Clone)]
struct Collected(Rc<RefCell<Vec<Diagnostic>>>);

impl DiagnosticSink for Collected {
    fn emit(&self, diagnostic: &Diagnostic) {
        self.0.borrow_mut().push(diagnostic.clone());
    }
}

impl Collected {
    #[doc = include_str!("okui_slot_projection.doc/saw_stray_outlet.md")]
    fn saw_stray_outlet(&self) -> bool {
        self.0
            .borrow()
            .iter()
            .any(|d| d.code.as_str() == "view.stray_slot_outlet")
    }
}

#[test]
fn a_stray_outlet_outside_a_component_never_becomes_a_node() {
    // The residue this file used to pin, closed. Projection runs over a
    // component's render output; a view that is nobody's render is never
    // projected, so a stray outlet used to land as an ordinary empty element that
    // laid out and painted. The reconciler now refuses to materialize one.
    let mut runtime = runtime_with_card();
    runtime.render_now(&View::single(ViewNode::slot("header")), &[]);

    let root = runtime.tree().root().expect("a root");
    assert!(
        runtime
            .tree()
            .get(root)
            .expect("live")
            .child_nodes()
            .is_empty(),
        "a stray outlet must not become a real node"
    );
}

#[test]
fn a_stray_outlet_is_reported_on_the_diagnostics_channel() {
    // Dropping it silently is the state this campaign keeps finding and removing:
    // the author's outlet vanishes and nothing says why. `float`, `@container` and
    // `::before` are refused *with a diagnostic*; so is this.
    //
    // It cannot be refused at compile time, and that is not a shortcut: a
    // component's `render` returns a `View` and `okui!` produces a `View`, so the
    // identical `okui! { <slot name="header"/> }` is correct in `render` and stray
    // one function away. Only the runtime knows who is projecting the view.
    let collected = Collected::default();
    let mut runtime = runtime_with_card();
    runtime.set_diagnostic_sink(Box::new(collected.clone()));
    runtime.render_now(&View::single(ViewNode::slot("header")), &[]);

    let reported = collected.0.borrow();
    let stray = reported
        .iter()
        .find(|d| d.code.as_str() == "view.stray_slot_outlet")
        .expect("a stray outlet is reported");
    assert_eq!(stray.severity, DiagnosticSeverity::Error);
    assert!(
        stray
            .message
            .contains("outside a component's render output"),
        "the message must say what is wrong: {}",
        stray.message
    );
}

#[test]
fn an_outlet_inside_a_components_render_is_not_reported_as_stray() {
    // The disjoint half, and the one that makes the check worth having rather than
    // a blanket refusal of the tag: the outlets `Card::render` declares are the
    // legitimate ones. A rule that fired on those would refuse §2.12 itself.
    let collected = Collected::default();
    let mut runtime = runtime_with_card();
    runtime.set_diagnostic_sink(Box::new(collected.clone()));
    runtime.render_now(
        &okui! { <Card title="Hi"><h1 slot="header">"T"</h1></Card> },
        &[],
    );

    assert!(
        !collected.saw_stray_outlet(),
        "a component's own outlets are not strays"
    );
    // And they still project, so this is not "quiet because nothing ran".
    assert_eq!(texts_in_region(&runtime, "region-header"), ["T"]);
}

#[test]
fn an_outlet_in_a_hosts_authored_children_is_a_stray_too() {
    // The case the root-only check must not miss. A component host's children are
    // cloned into the component's output by `project_slots` *un-projected* — that
    // is what makes them content rather than output — so an outlet nested in them
    // is one nothing will ever fill. It is in the desired view, so the root check
    // sees it, and the reconciler refuses it wherever it lands.
    let collected = Collected::default();
    let mut runtime = runtime_with_card();
    runtime.set_diagnostic_sink(Box::new(collected.clone()));
    runtime.render_now(
        &okui! { <Card title="Hi"><div slot="header"><slot name="inner"/></div></Card> },
        &[],
    );

    assert!(
        collected.saw_stray_outlet(),
        "an outlet inside a host's authored children is a stray"
    );
    let region = find_by_class(
        &runtime,
        runtime.tree().root().expect("a root"),
        "region-header",
    )
    .expect("the region is the component's own markup");
    let projected = runtime.tree().get(region).expect("live").child_nodes();
    assert_eq!(projected.len(), 1, "the assigned div still projects");
    assert!(
        runtime
            .tree()
            .get(projected[0])
            .expect("live")
            .child_nodes()
            .is_empty(),
        "the nested stray outlet is not under it"
    );
}

// --- The residue, pinned rather than hidden. -----------------------------

#[test]
fn a_stray_outlet_is_refused_silently_where_there_is_no_diagnostics_channel() {
    // The honest limit of the report. `FramePipeline` — public, in the prelude —
    // is the bare deterministic core: it holds no diagnostics sink and drives no
    // components, so *every* outlet reaching it is a stray and none of them is
    // reported. The refusal still holds, because it lives in the reconciler that
    // both paths share; only the telling is `UiRuntime`'s.
    let mut tree = RetainedTree::new();
    let mut scene = Scene::new();
    let ctx = FrameContext::new(
        FrameId::from_raw(1),
        FrameReason::UiCommit,
        Size::new(100.0, 100.0),
    );
    let _ = FramePipeline::new().run(
        &mut tree,
        &mut scene,
        ctx,
        &View::single(ViewNode::slot("header")),
        &StyleInputs::default(),
    );

    let root = tree.root().expect("a root");
    assert!(
        tree.get(root).expect("live").child_nodes().is_empty(),
        "the refusal is the reconciler's, so it holds without a runtime"
    );
}
