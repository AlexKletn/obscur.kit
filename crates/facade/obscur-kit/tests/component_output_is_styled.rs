//! Both halves of §4.15, on real frames: a component's rendered output is
//! **scoped by default**, an "explicitly open" component's output is styled by
//! the document exactly as authored content is, and the system default (UA)
//! sheet reaches inside either.
//!
//! # What this file was written from
//!
//! The external consumer audit's second-pass finding 2-1 — "a component-rendered
//! subtree is matched by no selector". Its control experiment is
//! [`the_documents_selectors_stop_at_a_scoped_components_boundary`] below, and it
//! reproduced exactly as reported: one screen, one sheet, one frame, two nodes
//! identical apart from provenance, and only the authored one is styled.
//!
//! It is not, however, matching that is broken. The mechanism is deliberate:
//! `orchestration::style_scoping::component_host` finds the nearest
//! `NodeFlags::COMPONENT_HOST` ancestor and the style-matching coordinator hands
//! any node that has one to `scoped_internal_declarations`, where only an
//! exported `::part()` crosses. That is §4.15's "component stylesheets are
//! scoped by default", and `spec_ch2_contracts.rs` pins it. (The audit's grep
//! looked for it under `crates/domain/obscur-style/src/matching/` and came up
//! empty because the boundary is a *retained-tree* fact, which the pure matcher
//! deliberately cannot see.)
//!
//! What was genuinely missing is everything the closing half implies but nothing
//! provided:
//!
//! * §4.15's other bullet — "**explicitly open/global scopes are possible**" —
//!   had no implementation, so no component could opt out;
//! * there is no producer for the `ComponentModule` cascade origin, so a
//!   component cannot ship a scoped sheet of its own either;
//! * the system default sheet was scoped out with the author sheets, so a
//!   `<button>` a component rendered was not a button.
//!
//! Together those made the tier unusable for anything visual, which is the
//! audit's verdict and it stands. `StyleScope::Open` and the UA crossing are
//! what close it; the default is untouched.
//!
//! # Why a computed style and not rendered text
//!
//! `okui_user_component.rs` and its siblings assert on rendered **text**, which
//! is why this survived: text proves a component mounted and that its props
//! decoded, and says nothing about whether the nodes it produced are visible to
//! the cascade. Every assertion here reads a computed value off the runtime.
#![forbid(unsafe_code)]

use obscur_kit::bootstrap::CssPipeline;
use obscur_kit::css::source::CssSource;
use obscur_kit::default_stylesheet::ua_stylesheet;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, InternedSymbol, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::style::property::PropertyId;
use obscur_kit::ui_ir::name_binding;
use obscur_kit::ui_macros::Contract;
use obscur_kit::view::component::{
    Component, ComponentContext, ComponentContract, ComponentFactory, ComponentInstance,
    ComponentRegistration, ComponentVersion, ErasedComponent, InstanceId,
};
use obscur_kit::view::view::{CLASS, TYPE, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// One class, one declaration, matched against two nodes that both carry it —
/// plus a descendant rule whose ancestor is always authored by the application
/// and whose subject may not be.
const STYLESHEET: &str = "\
.counter { background-color: rgb(30, 38, 54); }\n\
.shell .label { color: rgb(200, 10, 10); }\n\
";

/// What `.counter` paints.
const FILL: (f32, f32, f32) = (30.0 / 255.0, 38.0 / 255.0, 54.0 / 255.0);
/// What `.shell .label` inks.
const INK: (f32, f32, f32) = (200.0 / 255.0, 10.0 / 255.0, 10.0 / 255.0);
/// The colour the cascade lands on when nothing at all matched: the initial
/// value of both properties involved is transparent/black.
const NOTHING: (f32, f32, f32) = (0.0, 0.0, 0.0);
/// The UA sheet's `button { color: #2b2a27 }` ink.
const UA_BUTTON_INK: (f32, f32, f32) = (
    0x2b as f32 / 255.0,
    0x2a as f32 / 255.0,
    0x27 as f32 / 255.0,
);

/// An element with a CSS type and a class, so a type selector, a class selector
/// and a descendant selector all have something to bite on.
fn element(local_name: &str, class: &str) -> ViewNode {
    ViewNode::element(name_binding::CONTAINER)
        .with_prop(ViewProperty::new(TYPE, ViewValue::Text(local_name.into())))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}

/// `<div class="…">`.
fn div(class: &str) -> ViewNode {
    element("div", class)
}

/// The markup both the components and the authored control render:
/// `<div class="counter"><div class="label">…</div><button/></div>`.
fn body(label: &str) -> ViewNode {
    div("counter").with_children(ViewFragment::from_nodes(vec![
        div("label").with_children(ViewFragment::from_nodes(vec![ViewNode::text(label)])),
        element("button", "press"),
    ]))
}

// --- Two components, identical but for their declared scope. -------------

/// A component that says nothing about its style scope, so §4.15's default
/// applies and its output is encapsulated.
#[derive(Contract)]
#[contract(name = "ScopedCounter")]
struct ScopedProps {
    /// The text it renders, so "did not mount" reads differently from "was not
    /// styled".
    label: String,
}

/// The same component with `#[contract(style_scope = open)]` — §4.15's
/// "explicitly open/global scopes are possible", declared where the promise
/// about the internals is made.
#[derive(Contract)]
#[contract(name = "OpenCounter")]
#[contract(style_scope = open)]
struct OpenProps {
    /// See [`ScopedProps::label`].
    label: String,
}

/// A component that renders [`body`] and nothing else. Generic over its props so
/// the scoped and open variants are provably the same component but for the
/// contract.
struct Counter<P>(std::marker::PhantomData<P>);

impl Component for Counter<ScopedProps> {
    type Props = ScopedProps;

    fn render(&mut self, _ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![body(&props.label)]))
    }
}

impl Component for Counter<OpenProps> {
    type Props = OpenProps;

    fn render(&mut self, _ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![body(&props.label)]))
    }
}

#[derive(Debug)]
struct ScopedFactory;

impl ComponentFactory for ScopedFactory {
    fn component_id(&self) -> ComponentId {
        ScopedProps::contract().id
    }
    fn contract(&self) -> ComponentContract {
        ScopedProps::contract()
    }
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, ScopedProps::contract().id)
    }
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(Counter::<ScopedProps>(std::marker::PhantomData))
    }
}

#[derive(Debug)]
struct OpenFactory;

impl ComponentFactory for OpenFactory {
    fn component_id(&self) -> ComponentId {
        OpenProps::contract().id
    }
    fn contract(&self) -> ComponentContract {
        OpenProps::contract()
    }
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, OpenProps::contract().id)
    }
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(Counter::<OpenProps>(std::marker::PhantomData))
    }
}

// --- The screen and the frame. -------------------------------------------

/// `<div class="shell"><Counter label="…"/><div class="counter">…</div></div>`,
/// where `Counter` is whichever of the two components `kind` names.
///
/// The authored sibling is not scenery: it is the control. It carries the same
/// classes, the same nesting and the same `<button>`, so the only difference
/// between the two subtrees is which of them a component produced.
fn screen(kind: ComponentId) -> View {
    // The prop symbol is the interned prop *name* — the scheme
    // `#[derive(Contract)]` and both front ends compute.
    let component = ViewNode::element(kind).with_prop(ViewProperty::new(
        InternedSymbol::named("label"),
        ViewValue::Text("состояние".into()),
    ));
    View::from_fragment(ViewFragment::from_nodes(vec![div("shell").with_children(
        ViewFragment::from_nodes(vec![component, body("контроль")]),
    )]))
}

/// A runtime with both components registered, one frame of [`screen`] rendered,
/// and — when `ua` — the toolkit's UA default sheet installed.
fn settled(kind: ComponentId, ua: bool) -> UiRuntime {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 300.0));
    runtime
        .register_component(ComponentRegistration::new(
            Box::new(ScopedFactory),
            ComponentVersion::INITIAL,
        ))
        .expect("ScopedCounter registers");
    runtime
        .register_component(ComponentRegistration::new(
            Box::new(OpenFactory),
            ComponentVersion::INITIAL,
        ))
        .expect("OpenCounter registers");
    if ua {
        runtime.set_default_stylesheet(ua_stylesheet());
    }
    let mut css = CssPipeline::new();
    css.install(&mut runtime, &[CssSource::anonymous(STYLESHEET)]);
    let sheets: Vec<Stylesheet> = css.sheets().to_vec();
    let _ = runtime.render_now(&screen(kind), &sheets);
    runtime
}

fn colour(runtime: &UiRuntime, node: NodeId, property: PropertyId) -> (f32, f32, f32) {
    runtime
        .computed_style(node)
        .and_then(|style| style.get(property))
        .and_then(|value| value.as_color())
        .map(|c| (c.r, c.g, c.b))
        .expect("every element node resolves a computed colour")
}

/// The node at `path`, walking child indices from the root.
fn at(runtime: &UiRuntime, path: &[usize]) -> NodeId {
    let mut node = runtime.tree().root().expect("a root");
    for &index in path {
        node = runtime.tree().get(node).expect("a live node").child_nodes()[index];
    }
    node
}

/// The component's `div.counter`, its `div.label` and its `<button>`.
const INSIDE: (&[usize], &[usize], &[usize]) = (&[0, 0, 0], &[0, 0, 0, 0], &[0, 0, 0, 1]);
/// The authored control's same three nodes.
const CONTROL: (&[usize], &[usize], &[usize]) = (&[0, 1], &[0, 1, 0], &[0, 1, 1]);

// --- §4.15, half one: scoped by default. ---------------------------------

#[test]
fn the_documents_selectors_stop_at_a_scoped_components_boundary() {
    // The audit's control experiment, kept as the pin on the DEFAULT. A
    // component that declares no scope encapsulates its internals: neither the
    // class rule nor the descendant rule reaches in, while both apply to the
    // authored sibling that is identical in every other respect.
    let runtime = settled(ScopedProps::contract().id, false);

    assert_eq!(
        colour(
            &runtime,
            at(&runtime, CONTROL.0),
            PropertyId::BackgroundColor
        ),
        FILL,
        "the authored control is the baseline: `.counter` matches it"
    );
    assert_eq!(
        colour(&runtime, at(&runtime, CONTROL.1), PropertyId::Color),
        INK,
        "and so does `.shell .label`"
    );

    assert_eq!(
        colour(
            &runtime,
            at(&runtime, INSIDE.0),
            PropertyId::BackgroundColor
        ),
        NOTHING,
        "`.counter` does not reach a scoped component's internals (§4.15)"
    );
    assert_eq!(
        colour(&runtime, at(&runtime, INSIDE.1), PropertyId::Color),
        NOTHING,
        "nor does a descendant selector rooted outside the component"
    );
}

// --- §4.15, half two: explicitly open. -----------------------------------

#[test]
fn an_open_component_is_styled_by_the_documents_sheets_exactly_as_authored_content_is() {
    // The fix, stated as the property the audit demanded: two nodes identical
    // apart from provenance compute the same style. `OpenCounter` differs from
    // `ScopedCounter` by one contract attribute and nothing else — same render
    // function, same markup, same screen.
    let runtime = settled(OpenProps::contract().id, false);

    assert_eq!(
        colour(
            &runtime,
            at(&runtime, INSIDE.0),
            PropertyId::BackgroundColor
        ),
        colour(
            &runtime,
            at(&runtime, CONTROL.0),
            PropertyId::BackgroundColor
        ),
        "an open component's node and its authored twin must compute the same fill"
    );
    assert_eq!(
        colour(
            &runtime,
            at(&runtime, INSIDE.0),
            PropertyId::BackgroundColor
        ),
        FILL,
        "...and that fill is the rule's, not both-of-them-unstyled"
    );
}

#[test]
fn a_descendant_selector_climbs_from_the_document_into_an_open_component() {
    // `.shell .label` crosses the boundary: the ancestor is authored by the
    // application, the subject is rendered by the component. The matcher's
    // ancestor walk climbs the retained tree, and an open host is not a wall.
    let runtime = settled(OpenProps::contract().id, false);

    assert_eq!(
        colour(&runtime, at(&runtime, INSIDE.1), PropertyId::Color),
        INK,
        "the component-rendered `.label` inherits the document's ancestry"
    );
}

// --- The system default sheet is not an author sheet. --------------------

#[test]
fn the_system_default_sheet_reaches_inside_a_scoped_component() {
    // The audit's `button.press` observation: a `<button>` a component rendered
    // computed `color: rgb(0,0,0)` — the initial value — where the same
    // `<button>` outside got the UA sheet's ink. §4.15 scopes *authored*
    // stylesheets; the UA sheet is the built-in definition of what a `<button>`
    // is, sits at the lowest cascade origin, and encapsulates nothing by being
    // withheld. A scoped component still keeps the AUTHOR sheets out — that is
    // the assertion pair below.
    let runtime = settled(ScopedProps::contract().id, true);

    assert_eq!(
        colour(&runtime, at(&runtime, CONTROL.2), PropertyId::Color),
        UA_BUTTON_INK,
        "the authored `<button>` gets the UA sheet's ink"
    );
    assert_eq!(
        colour(&runtime, at(&runtime, INSIDE.2), PropertyId::Color),
        UA_BUTTON_INK,
        "so does a `<button>` a scoped component rendered — it is still a button"
    );
    assert_eq!(
        colour(
            &runtime,
            at(&runtime, INSIDE.0),
            PropertyId::BackgroundColor
        ),
        NOTHING,
        "the author sheet is still contained: opening the UA sheet is not opening the scope"
    );
}

// --- Composition: opening yourself is not opening somebody else. ---------

/// A scoped wrapper that renders an `<OpenCounter/>` inside its own output, so
/// the open component sits *below* a closed boundary.
#[derive(Contract)]
#[contract(name = "ScopedWrapper")]
struct WrapperProps {}

struct Wrapper;

impl Component for Wrapper {
    type Props = WrapperProps;

    fn render(&mut self, _ctx: &mut ComponentContext<'_>, _props: &Self::Props) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![div("shell").with_children(
            ViewFragment::from_nodes(vec![ViewNode::element(OpenProps::contract().id).with_prop(
                ViewProperty::new(
                    InternedSymbol::named("label"),
                    ViewValue::Text("вложенный".into()),
                ),
            )]),
        )]))
    }
}

#[derive(Debug)]
struct WrapperFactory;

impl ComponentFactory for WrapperFactory {
    fn component_id(&self) -> ComponentId {
        WrapperProps::contract().id
    }
    fn contract(&self) -> ComponentContract {
        WrapperProps::contract()
    }
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, WrapperProps::contract().id)
    }
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(Wrapper)
    }
}

#[test]
fn an_open_component_nested_inside_a_scoped_one_is_still_contained() {
    // The composition rule, and the reason an open host is *transparent* to the
    // scope walk rather than *terminal*: declaring yourself open lets the
    // document reach you, it must not let you punch a hole through the
    // encapsulation of a component that wrapped you.
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 300.0));
    for factory in [
        ComponentRegistration::new(Box::new(OpenFactory), ComponentVersion::INITIAL),
        ComponentRegistration::new(Box::new(WrapperFactory), ComponentVersion::INITIAL),
    ] {
        runtime.register_component(factory).expect("registers");
    }
    let mut css = CssPipeline::new();
    css.install(&mut runtime, &[CssSource::anonymous(STYLESHEET)]);
    let sheets: Vec<Stylesheet> = css.sheets().to_vec();
    let view = View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::element(
        WrapperProps::contract().id,
    )]));
    let _ = runtime.render_now(&view, &sheets);

    // Wrapper host → its `div.shell` → the OpenCounter host → `div.counter`.
    let inner = at(&runtime, &[0, 0, 0, 0]);
    assert_eq!(
        colour(&runtime, inner, PropertyId::BackgroundColor),
        NOTHING,
        "the open component is inside ScopedWrapper's boundary, so the document \
         still does not reach it"
    );
}
