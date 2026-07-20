//! End-to-end: an authored `<Card/>` **is** the author's component (spec §2.13,
//! §2.14) — on both front ends.
//!
//! # What used to happen
//!
//! `name_binding::component_id_for` resolved every tag it did not recognise to
//! the generic `Container` primitive. So `<Card/>` was not a mis-checked
//! component and not a rejected one: it was a **box**. No diagnostic, no error,
//! no mount — an author wrote a component and got an empty div. §2.13's contract
//! validation "skips any tag with no registered contract" was checking nothing
//! because the tag was never going to be a component in the first place.
//!
//! # What closes it
//!
//! The id a component tag binds to is derived from its **name**
//! ([`name_binding::user_component_id`]), which is the very scheme
//! `#[derive(Contract)]` already computes `ComponentContract::id` with. The
//! registry (`ComponentRegistry`) is a *runtime* value holding
//! `Box<dyn ComponentFactory>` trait objects and cannot be consulted by a code
//! generator that runs while the host crate is still compiling — but it does not
//! need to be, because both sides can compute the same id from the same string.
//! `derived_contract_id_agrees_with_the_authored_tag` below is the drift guard on
//! exactly that agreement.
//!
//! The runtime half was already real and already tested (`ComponentStore` mounts
//! an instance for any node whose `ComponentId` is registered and splices its
//! `render` output beneath it); the front ends were emitting the wrong id into
//! it.
//!
//! # Why every end-to-end test here carries a sibling box
//!
//! Because the first draft of them did not, and passed with the fix reverted.
//! The derive resolves a name through the *same* `component_id_for` the front
//! ends do — so in a build where `Card` falls back to `Container`, the host
//! registers Card under the container id, `<Card/>` emits the container id, they
//! agree, and the component mounts and renders "card:Hi" exactly as it should.
//! What that build also does is mount a Card for every `<div>` on the page.
//!
//! Agreement between the two sides is therefore not the property under test —
//! *identity* is. The `<div>`/`<span>` siblings are what tell the two apart:
//! exactly one of the three nodes may mount.
//!
//! # What is still open, and is not claimed here
//!
//! * **An unregistered component tag is still silent.** `<Crad/>` (a typo) now
//!   binds to `user_component_id("Crad")`, nothing is registered under it, and it
//!   mounts nothing — no diagnostic. It is not even visibly different: nothing in
//!   matching, cascade, layout or paint reads a `ComponentId`, so it still renders
//!   as the box it silently rendered as before. The file front end *could*
//!   diagnose this (its `Compiler` takes a `ContractRegistry`); the `okui!` macro
//!   provably cannot (`CardProps::contract_surface()` is a function that runs when
//!   the built crate runs, and the macro runs while that crate is still being
//!   compiled), so the rule cannot be shared and is not added to one front end
//!   alone. See `an_unregistered_component_tag_mounts_nothing`, which pins the
//!   residue rather than hiding it.
//! * **A component that declares no outlet still drops its authored children**,
//!   and this file's `Card` is one: it declares no slot and renders no
//!   `ViewNode::slot(…)`, so the `<span slot="header"/>` beneath it reaches no
//!   position and does not appear. That is §2.12 working, not §2.12 missing — a
//!   slot is a declaration, and Card declares none.
//!   `authored_children_under_a_component_that_declares_no_slot_are_dropped` pins
//!   it; `okui_slot_projection.rs` is where a component that *does* declare one
//!   receives its content.
#![forbid(unsafe_code)]

use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::okui_compiler::bridge::UiIrViewBridge;
use obscur_kit::okui_compiler::codegen::ViewCodegen;
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
use obscur_kit::view::view::{View, ViewFragment, ViewNode};
use obscur_kit::widgets::button::Button;

// --- The author's component: a props struct, a component, a factory. ---

/// `Card`'s typed props. The `#[contract(name = "Card")]` override is what ties
/// the Rust type to the authored tag — the derive hashes *that* string into the
/// component id, and so does the front end when it meets `<Card/>`.
#[derive(Contract)]
#[contract(name = "Card")]
struct CardProps {
    /// A required prop, decoded from the authored `title="…"`.
    title: String,
}

/// The component itself: renders one text node naming its decoded prop, so the
/// assertion below can tell "the component ran" from "a box appeared".
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

/// A headless runtime with `Card` registered — the host half of §2.14's "static
/// registration is the default".
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

/// The text the component rendered underneath the node at `path` (child indices
/// from the root), or `None` when that node has no text child — which is exactly
/// what an unmounted box looks like.
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
    let generated = ViewCodegen::document_to_view_expr(&ir);

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
