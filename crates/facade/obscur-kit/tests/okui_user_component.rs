#![doc = include_str!("okui_user_component.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::okui_compiler::bridge::UiIrViewBridge;
use obscur_kit::okui_compiler::codegen::{CodegenRoot, ViewCodegen};
use obscur_kit::okui_compiler::compile::Compiler;
use obscur_kit::okui_syntax::ast::Document;
use obscur_kit::okui_syntax::parser::Parser;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::ui_ir::name_binding::{self, TagPosition};
use obscur_kit::ui_macros::{Contract, okui};
use obscur_kit::view::component::{
    Component, ComponentContext, ComponentContract, ComponentFactory, ComponentInstance,
    ComponentRegistration, ComponentVersion, ErasedComponent, InstanceId,
};
use obscur_kit::view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};
use obscur_kit::widgets::button::Button;

#[doc = include_str!("okui_user_component.doc/CardProps.md")]
#[derive(Contract)]
#[contract(name = "Card")]
struct CardProps {
    #[doc = include_str!("okui_user_component.doc/title.md")]
    title: String,
}

#[doc = include_str!("okui_user_component.doc/Card.md")]
struct Card;

impl Component for Card {
    type Props = CardProps;

    fn render(&mut self, _ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::text(format!(
            "card:{}",
            props.title
        ))]))
    }
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

#[doc = include_str!("okui_user_component.doc/runtime_with_card.md")]
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

#[doc = include_str!("okui_user_component.doc/rendered_text_at.md")]
fn rendered_text_at(runtime: &UiRuntime, path: &[usize]) -> Option<String> {
    let mut node = runtime.tree().root()?;
    for &index in path {
        node = *runtime.tree().get(node)?.child_nodes().get(index)?;
    }
    let child = *runtime.tree().get(node)?.child_nodes().first()?;
    match runtime.tree().get(child)?.kind() {
        obscur_kit::tree::node::NodeKind::Text { content } => Some(content.to_string()),
        _ => None,
    }
}

#[doc = include_str!("okui_user_component.doc/compile.md")]
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

// --- The agreement the whole fix rests on. -------------------------------

#[test]
fn derived_contract_id_agrees_with_the_authored_tag_and_is_not_the_box() {
    // THE drift guard. `#[derive(Contract)]` computes the id at expansion time
    // from the name string; the front end computes it at lowering time from the
    // tag string. Nothing forces them to agree except this — and if they ever
    // disagree, a registered component silently never mounts, which is precisely
    // the class of bug this file exists to close.
    assert_eq!(
        CardProps::contract().id,
        name_binding::component_id_for("Card", TagPosition::Component),
        "the derive's id and the front end's tag binding must be the same value"
    );
    // Agreement alone is not the property. The derive resolves the name through
    // the same `component_id_for`, so the two sides agree even in a build where
    // BOTH resolve `Card` to the generic box — and then every `<div>` in the
    // document is a Card too. Identity is the property; this is the half that
    // says so.
    assert_ne!(
        CardProps::contract().id,
        name_binding::CONTAINER,
        "a component whose id is the container primitive is not a component"
    );
}

#[test]
fn a_user_component_tag_no_longer_binds_to_the_generic_box() {
    // The regression, stated directly: `Card` is not `div`.
    assert_ne!(
        name_binding::component_id_for("Card", TagPosition::Component),
        name_binding::CONTAINER,
        "<Card/> must not resolve to the generic Container primitive"
    );
}

#[test]
fn builtins_and_elements_still_bind_exactly_as_before() {
    // The disjoint half. Making user components real must not move a single
    // built-in or element binding: a widget tag is still its widget, and every
    // element tag — known or not — is still the generic styleable box.
    assert_eq!(
        name_binding::component_id_for("Button", TagPosition::Component),
        Button::COMPONENT_ID,
        "a built-in widget wins over the name hash"
    );
    assert_eq!(
        name_binding::component_id_for("div", TagPosition::Element),
        name_binding::CONTAINER
    );
    assert_eq!(
        name_binding::component_id_for("something-custom", TagPosition::Element),
        name_binding::CONTAINER
    );
}

#[test]
fn a_user_component_id_can_never_collide_with_a_builtin() {
    // Not "unlikely" — structural. A user id carries the top bit; every built-in
    // handle is a small low-range constant, the same split `fallback_symbol`
    // already uses for attribute/event names.
    assert!(name_binding::user_component_id("Card").raw() & 0x8000_0000_0000_0000 != 0);
    for builtin in [
        name_binding::CONTAINER,
        name_binding::IMAGE,
        name_binding::SCROLL,
        name_binding::CANVAS,
        name_binding::PORTAL,
        name_binding::LAYER,
        name_binding::FOREIGN_SURFACE,
        Button::COMPONENT_ID,
    ] {
        assert_eq!(
            builtin.raw() & 0x8000_0000_0000_0000,
            0,
            "{builtin:?} must stay outside the user-component namespace"
        );
    }
}

// --- Front end 1: the `okui!` macro (§2.8, the primary surface). ---------

#[test]
fn the_macro_renders_a_registered_component_and_only_that_tag() {
    // THE end-to-end slice for the inline front end: `okui!` lowers `<Card/>` to
    // ordinary `View`-building Rust naming the real component id; the runtime
    // mounts the registered kind, decodes `title` from the authored property, and
    // splices the component's own output beneath the node.
    //
    // The `<div>` and `<span>` siblings are not scenery — they are what makes
    // this test discriminate. Asserting only that `<Card/>` renders "card:Hi"
    // passes in a build where `Card`, `div` and `span` ALL resolve to the
    // container primitive: the host registers Card under that id, the tag emits
    // it, and they agree. Agreement is not identity. Exactly one of these three
    // nodes may mount.
    let mut runtime = runtime_with_card();
    let view: View = okui! {
        <div>
            <Card title="Hi"/>
            <span/>
        </div>
    };
    runtime.render_now(&view, &[]);

    assert_eq!(
        runtime.mounted_component_count(),
        1,
        "only <Card/> names the registered component — a <div>/<span> is a box"
    );
    assert_eq!(
        rendered_text_at(&runtime, &[0, 0]).as_deref(),
        Some("card:Hi"),
        "the component rendered, and its decoded prop reached render"
    );
    assert_eq!(
        rendered_text_at(&runtime, &[0, 1]),
        None,
        "the <span> sibling stayed an empty box"
    );
}

// --- Front end 2: the `.okui` file (§2.9), on both of its paths. ---------

#[test]
fn the_file_front_end_bridges_a_registered_component_and_only_that_tag() {
    // The same document through the real parse → compile → lower pipeline and the
    // runtime bridge. Both front ends gained the change together, which is the
    // house rule the `<VirtualFor>`-was-`<For>` drift was written to enforce.
    // The sibling box discriminates here for the same reason as above.
    let ir = compile(r#"<div><Card title="Hi"/><span/></div>"#);
    let view = UiIrViewBridge::document_to_view(&ir);

    let mut runtime = runtime_with_card();
    runtime.render_now(&view, &[]);

    assert_eq!(
        runtime.mounted_component_count(),
        1,
        "only <Card/> names the registered component"
    );
    assert_eq!(
        rendered_text_at(&runtime, &[0, 0]).as_deref(),
        Some("card:Hi")
    );
    assert_eq!(rendered_text_at(&runtime, &[0, 1]), None);
}

#[test]
fn the_codegen_path_emits_the_components_real_id() {
    // The compile-time path emits Rust text, so this is the only place it can be
    // checked without a second `rustc`: the emitted source must carry Card's real
    // handle and not the Container constant.
    let ir = compile(r#"<Card title="Hi"/>"#);
    let generated = ViewCodegen::document_to_view_expr(&ir, &CodegenRoot::facade());

    let card = CardProps::contract().id.raw();
    assert!(
        generated.contains(&format!("from_raw({card:#x})")),
        "generated source must name Card's real id:\n{generated}"
    );
    let container = name_binding::CONTAINER.raw();
    assert!(
        !generated.contains(&format!("from_raw({container:#x})")),
        "generated source must not fall back to Container:\n{generated}"
    );
}

// --- The residue, pinned rather than hidden. ----------------------------

#[test]
fn an_unregistered_component_tag_mounts_nothing() {
    // Honest about what is NOT fixed. A typo'd tag binds to its own id, nothing
    // is registered under it, and it renders as an empty node — no diagnostic.
    //
    // And it is barely better than before: no part of styling, layout or paint
    // reads a `ComponentId` (only `ComponentStore::mount` does), so `<Crad
    // class="x"/>` still cascades, lays out and paints exactly like the box it
    // used to silently be. The one thing that changed for it is that it can no
    // longer collide with a component registered under the container id. This is
    // not fail-closed, and this test says so out loud rather than letting the
    // file's title imply otherwise.
    let mut runtime = runtime_with_card();
    let view: View = okui! { <Crad title="Hi"/> };
    runtime.render_now(&view, &[]);

    assert_eq!(
        runtime.mounted_component_count(),
        0,
        "nothing is registered under `Crad`"
    );
    assert_eq!(
        rendered_text_at(&runtime, &[0]),
        None,
        "an unregistered component tag is still silent — see the module docs"
    );
}

#[test]
fn authored_children_under_a_component_that_declares_no_slot_are_dropped() {
    // The other side of §2.12, pinned. This `Card` declares no slot and renders no
    // outlet, so there is no position for the `<span slot="header"/>` to reach and
    // it does not appear. The component's own output is all that lands.
    //
    // This is not the old projection gap left standing: a Card that declares
    // `header` and renders `ViewNode::slot("header")` receives the span, which is
    // what `okui_slot_projection.rs` drives. Content goes where a component says
    // it goes, and nowhere by default — §2.12 declares *named* slots and says
    // nothing about an unnamed one.
    let mut runtime = runtime_with_card();
    let view: View = okui! { <Card title="Hi"><span slot="header"/></Card> };
    runtime.render_now(&view, &[]);

    assert_eq!(
        rendered_text_at(&runtime, &[0]).as_deref(),
        Some("card:Hi"),
        "the component's own output is what lands under its node"
    );
    let root = runtime.tree().root().expect("a root");
    let card = runtime.tree().get(root).expect("live").child_nodes()[0];
    assert_eq!(
        runtime.tree().get(card).expect("live").child_nodes().len(),
        1,
        "a component declaring no slot has nowhere to put an authored child"
    );
}

// --- The other half of "a component works": it can be styled. -----------

#[doc = include_str!("okui_user_component.doc/PanelProps.md")]
#[derive(Contract)]
#[contract(name = "Panel")]
#[contract(style_scope = open)]
struct PanelProps {
    #[doc = include_str!("okui_user_component.doc/title_2.md")]
    title: String,
}

struct Panel;

impl Component for Panel {
    type Props = PanelProps;

    fn render(&mut self, _ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(name_binding::CONTAINER)
                .with_prop(ViewProperty::new(
                    obscur_kit::view::view::CLASS,
                    ViewValue::Text("panel-body".into()),
                ))
                .with_children(ViewFragment::from_nodes(vec![ViewNode::text(format!(
                    "panel:{}",
                    props.title
                ))])),
        ]))
    }
}

#[derive(Debug)]
struct PanelFactory;

impl ComponentFactory for PanelFactory {
    fn component_id(&self) -> ComponentId {
        PanelProps::contract().id
    }
    fn contract(&self) -> ComponentContract {
        PanelProps::contract()
    }
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, PanelProps::contract().id)
    }
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(Panel)
    }
}

#[doc = include_str!("okui_user_component.doc/runtime_with_panel.md")]
fn runtime_with_panel() -> UiRuntime {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));
    runtime
        .register_component(ComponentRegistration::new(
            Box::new(PanelFactory),
            ComponentVersion::INITIAL,
        ))
        .expect("Panel registers");
    runtime
}

#[doc = include_str!("okui_user_component.doc/PANEL_CSS.md")]
const PANEL_CSS: &str = ".panel-body { background-color: rgb(30, 38, 54); }";
#[doc = include_str!("okui_user_component.doc/PANEL_FILL.md")]
const PANEL_FILL: (f32, f32, f32) = (30.0 / 255.0, 38.0 / 255.0, 54.0 / 255.0);

#[doc = include_str!("okui_user_component.doc/component_output_fill.md")]
fn component_output_fill(runtime: &mut UiRuntime, view: &View, path: &[usize]) -> (f32, f32, f32) {
    let mut css = obscur_kit::bootstrap::CssPipeline::new();
    css.install(
        runtime,
        &[obscur_kit::css::source::CssSource::anonymous(PANEL_CSS)],
    );
    let sheets = css.sheets().to_vec();
    runtime.render_now(view, &sheets);

    let mut node = runtime.tree().root().expect("a root");
    for &index in path {
        node = runtime.tree().get(node).expect("a live node").child_nodes()[index];
    }
    runtime
        .computed_style(node)
        .and_then(|style| style.get(obscur_kit::style::property::PropertyId::BackgroundColor))
        .and_then(|value| value.as_color())
        .map(|c| (c.r, c.g, c.b))
        .expect("an element node resolves a computed background colour")
}

#[test]
fn the_macro_renders_a_component_whose_output_the_document_styles() {
    // The habit, on the inline front end. `<span class="panel-body"/>` is the
    // control: the same class, authored inline, one frame, one sheet. Provenance
    // must not decide whether a rule applies.
    let mut runtime = runtime_with_panel();
    let view: View = okui! {
        <div>
            <Panel title="Hi"/>
            <span class="panel-body"/>
        </div>
    };

    // The component's own element is the first child of the `<Panel/>` node.
    assert_eq!(
        component_output_fill(&mut runtime, &view, &[0, 0, 0]),
        PANEL_FILL,
        "`.panel-body` must reach the element the component rendered"
    );
    assert_eq!(
        component_output_fill(&mut runtime, &view, &[0, 1]),
        PANEL_FILL,
        "...the same value the authored control computes"
    );
    assert_eq!(
        rendered_text_at(&runtime, &[0, 0, 0]).as_deref(),
        Some("panel:Hi"),
        "and the component still rendered — a styled empty box is not the claim"
    );
}

#[test]
fn the_file_front_end_renders_a_component_whose_output_the_document_styles() {
    // The same property through the real parse → compile → lower → bridge
    // pipeline, because both front ends gained the change together and a
    // regression in either is a regression in the tier.
    let ir = compile(r#"<div><Panel title="Hi"/><span class="panel-body"/></div>"#);
    let view = UiIrViewBridge::document_to_view(&ir);
    let mut runtime = runtime_with_panel();

    assert_eq!(
        component_output_fill(&mut runtime, &view, &[0, 0, 0]),
        PANEL_FILL,
        "`.panel-body` must reach the element the component rendered"
    );
    assert_eq!(
        component_output_fill(&mut runtime, &view, &[0, 1]),
        PANEL_FILL,
        "...the same value the authored control computes"
    );
}
