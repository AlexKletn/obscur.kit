//! End-to-end: an authored tag is a **base element** — it carries its own local
//! name, so a §4.13 type selector matches it — and `<button>` is one of them
//! (spec §4.13, §2.8).
//!
//! # What this closes
//!
//! An authored element used to reach the tree carrying a `ComponentId` and
//! nothing else, and a `ComponentId` is not a selector input: no part of matching
//! reads one. So `<button>`, `<div>` and `<td>` were three *anonymous* boxes, and
//! every type-selector rule in existence — including the whole of the facade's UA
//! stylesheet, which is written in `button { … }` / `input { … }` / `td { … }` and
//! nothing else — matched none of them. Only the Rust widgets were styleable,
//! because each sets the reserved `TYPE` property itself
//! (`obscur_widgets::Button::view`: "Stable CSS type / local name so a UA (or
//! author) `button { … }` type selector matches this control").
//!
//! Two things changed, both in the frontend:
//!
//! 1. `name_binding::local_name_for` — an *element*-position tag is also the
//!    node's local name, emitted as the reserved `TYPE` property by both lowering
//!    paths (`ViewCodegen::emit_element` and `UiIrViewBridge::lower_element`), so
//!    the compiled `okui!` path and the interpreted `.okui` path agree.
//! 2. the lowercase `button` tag binds to `Button::COMPONENT_ID`, the way `img`
//!    and `Image` already were one handle. HTML has a `<button>`; so does this.
//!
//! # What an authored `<button>` is, and is not
//!
//! It **is** an element with the button widget's component id, the author's own
//! children, the author's `on:click` binding (which the runtime dispatches off the
//! retained node's event binding, not off any component), the local name
//! `button` — and therefore the UA sheet's padding/border/background, and the
//! `SemanticRole::Button` the accessibility tree derives from a local name.
//!
//! It is **not** a live `obscur_widgets::Button`: nothing registers a factory for
//! the built-in widget ids by default (`UiRuntime::components` is "empty and inert
//! until a host calls `register_component`"), so the widget's own view — its
//! pressed state, its synthesized label child, its `:disabled` attribute — does
//! not run. The id is the handle a host *could* register under; it is not a
//! mounting.
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;

use accesskit::Role;
use obscur_accessibility_accesskit::bridge::AccessKitBridge;
use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{InternedSymbol, RuntimeId};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::okui_compiler::bridge::UiIrViewBridge;
use obscur_kit::okui_compiler::compile::Compiler;
use obscur_kit::okui_syntax::ast::Document;
use obscur_kit::okui_syntax::parser::Parser;
use obscur_kit::primitives::Container;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::orchestration::NodeInspection;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::ui_ir::name_binding::{self, TagPosition};
use obscur_kit::ui_macros::okui;
use obscur_kit::view::view::View;
use obscur_kit::widgets::button::Button;

/// Compiles authored CSS the way an application does.
fn stylesheet(css: &str) -> Stylesheet {
    let parsed = StylesheetParser::new(&CssSource::anonymous(css)).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// Renders `view` at 300x200 under `sheets` and returns the inspected tree.
fn render(view: &View, sheets: &[Stylesheet]) -> Vec<NodeInspection> {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 200.0));
    runtime.render_now(view, sheets);
    runtime.inspect_snapshot().nodes
}

/// The single inspected node carrying `class`.
fn node<'a>(nodes: &'a [NodeInspection], class: &str) -> &'a NodeInspection {
    let mut found = nodes
        .iter()
        .filter(|node| node.classes.iter().any(|name| name == class));
    let node = found.next().unwrap_or_else(|| panic!("no .{class} node"));
    assert!(found.next().is_none(), "more than one .{class} node");
    node
}

/// The border-box width of the `.class` node, or 0 if it was not laid out.
fn width(nodes: &[NodeInspection], class: &str) -> f32 {
    node(nodes, class).bounds.map_or(0.0, |r| r.size.width)
}

/// Lowers `source` through the **interpreted** `.okui` path — parse, compile,
/// `UiIrViewBridge` — the other of the two lowering paths.
fn interpreted(source: &str) -> View {
    let parsed = Parser::new().parse(source);
    let document = Document::cast(parsed.tree().root().clone()).expect("a document");
    let result = Compiler::new("screen.okui").compile(&document);
    assert!(
        !result.has_errors(),
        "diagnostics: {:?}",
        result.diagnostics()
    );
    UiIrViewBridge::document_to_view(result.document().expect("a lowered document"))
}

#[test]
fn both_spellings_of_button_are_the_shipped_widgets_id() {
    // Cross-crate, so this is the real widget constant and not the frontend's
    // mirror of it: `<button>` and `<Button/>` are one control, exactly as `<img>`
    // and `<Image/>` are one primitive. The lowercase spelling used to name no
    // built-in, so it fell through to the generic box.
    assert_eq!(
        name_binding::component_id_for("button", TagPosition::Element),
        Button::COMPONENT_ID
    );
    assert_eq!(
        name_binding::component_id_for("Button", TagPosition::Component),
        Button::COMPONENT_ID
    );
    assert_ne!(
        name_binding::component_id_for("button", TagPosition::Element),
        Container::COMPONENT_ID
    );
}

#[test]
fn an_authored_button_reaches_the_tree_named_button() {
    // The local name is what the matcher reads as an element's type, and it
    // arrives from the tag with no author involvement.
    let view = okui! { <button class="btn">"OK"</button> };
    let nodes = render(&view, &[]);
    let button = node(&nodes, "btn");
    assert_eq!(button.local_name.as_deref(), Some("button"));
    assert_eq!(button.component, Some(Button::COMPONENT_ID));
}

#[test]
fn a_div_is_still_a_container_and_now_says_so() {
    // The generic box did not become anything else — it just stopped being
    // anonymous, which is what makes `div { … }` a usable selector.
    let view = okui! { <div class="box"/> };
    let nodes = render(&view, &[]);
    let div = node(&nodes, "box");
    assert_eq!(div.local_name.as_deref(), Some("div"));
    assert_eq!(div.component, Some(Container::COMPONENT_ID));
}

#[test]
fn a_type_selector_matches_an_authored_button() {
    // THE claim. `button { … }` — no class, no id — sizes the authored element,
    // and the `<div>` beside it is untouched, so this is the *type* matching and
    // not a rule that reaches everything.
    let view = okui! {
        <div class="root">
            <button class="btn">"OK"</button>
            <div class="plain"/>
        </div>
    };
    let css = "button { width: 120px; height: 40px } .plain { width: 60px }";
    let nodes = render(&view, &[stylesheet(css)]);
    assert_eq!(width(&nodes, "btn"), 120.0);
    assert_eq!(
        node(&nodes, "btn").bounds.map(|r| r.size.height),
        Some(40.0)
    );
    assert_eq!(width(&nodes, "plain"), 60.0);
}

#[test]
fn the_interpreted_okui_path_names_the_same_element() {
    // The two lowering paths must not disagree about what an element *is*: a
    // compiled screen and an interpreted one are the same document. This is the
    // `.okui` file path — parser, compiler, `UiIrViewBridge` — matched by the same
    // type selector as the `okui!` expansion above.
    let view = interpreted("<div class=\"root\"><button class=\"btn\">\"OK\"</button></div>");
    let nodes = render(&view, &[stylesheet("button { width: 120px }")]);
    let button = node(&nodes, "btn");
    assert_eq!(button.local_name.as_deref(), Some("button"));
    assert_eq!(button.component, Some(Button::COMPONENT_ID));
    assert_eq!(width(&nodes, "btn"), 120.0);
}

#[test]
fn an_authored_type_attribute_wins_the_overloaded_slot() {
    // `TYPE` is one symbol carrying two things — the CSS local name and HTML's own
    // `<button type="submit">` — and the author owns it. So the node's type is
    // `submit`, `button { … }` no longer matches it, and `submit { … }` does. That
    // is the price of the overload, stated rather than hidden: an author who wants
    // both writes a class.
    let view = okui! {
        <div class="root">
            <button type="submit" class="btn">"OK"</button>
        </div>
    };
    let nodes = render(&view, &[stylesheet("button { width: 120px }")]);
    assert_eq!(node(&nodes, "btn").local_name.as_deref(), Some("submit"));
    assert_ne!(width(&nodes, "btn"), 120.0);

    let matched = render(&view, &[stylesheet("submit { width: 90px }")]);
    assert_eq!(width(&matched, "btn"), 90.0);
}

#[test]
fn an_authored_buttons_click_still_reaches_its_command() {
    // The other half of "what an authored `<button>` gets": the click. The runtime
    // resolves a click against the retained node's own event binding, not against a
    // mounted component, so an authored button dispatches even though nothing
    // registers a factory for the widget id — what does *not* run is the widget's
    // own view (its pressed state, its synthesized label, its `:disabled`).
    let view = okui! { <button class="btn" on:click={save}>"OK"</button> };
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 200.0));
    runtime.render_now(&view, &[stylesheet(".btn { width: 100px; height: 40px }")]);

    let at = Point::new(50.0, 20.0);
    let _ = runtime.handle_pointer(
        PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerEventKind::Down,
            at,
        )
        .as_trusted(),
    );
    let dispatch = runtime.handle_pointer(
        PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerEventKind::Up,
            at,
        )
        .as_trusted(),
    );
    assert_eq!(dispatch.command, Some(InternedSymbol::named("save")));
}

#[test]
fn an_authored_button_is_announced_as_a_button() {
    // The other consumer of a local name, and the second thing markup gained for
    // free: `SemanticBuilder::role_for_local_name` derives a §7.1 role from the
    // element's type, so an authored `<button>` is now announced as a button where
    // an anonymous box was announced as a plain group.
    let view = okui! { <button>"Save"</button> };
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 200.0));
    let bridge = Rc::new(RefCell::new(AccessKitBridge::new()));
    runtime.set_accessibility_bridge(bridge.clone());
    let _ = runtime.render_now(&view, &[]);

    let bridge = bridge.borrow();
    let update = bridge.last_tree_update().expect("a published tree");
    let (_, button) = update
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some("Save"))
        .cloned()
        .expect("the labelled node reached the bridge");
    assert_eq!(button.role(), Role::Button);
}

#[test]
fn a_slot_outlet_is_not_a_named_element() {
    // The one element-position tag that gets no name. An outlet is a §2.12
    // *position*, substituted away before reconcile, and an authored
    // `<slot name="header"/>` must stay byte-identical to the
    // `ViewNode::slot("header")` a component's render builds — one outlet with two
    // spellings, not two outlets. A local name on one of them would split the pair.
    let view: View = okui! { <slot name="header"/> };
    assert_eq!(
        view.root().nodes(),
        [obscur_kit::view::view::ViewNode::slot("header")]
    );
}

#[test]
fn the_ua_stylesheet_reaches_an_authored_button() {
    // The consequence worth stating: the facade's UA sheet is written entirely in
    // type selectors, so making authored elements nameable is what turned it on for
    // authored markup. A bare `<button>` now gets the default control look
    // (padding 11px 20px + a 1.5px border around its label) where it used to get
    // nothing at all; the `<div>` beside it still gets no UA rule.
    let view = okui! {
        <div class="root">
            <button class="btn">"OK"</button>
            <div class="plain"/>
        </div>
    };
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 200.0));
    runtime.set_default_stylesheet(obscur_kit::ua_stylesheet());
    runtime.render_now(&view, &[]);
    let nodes = runtime.inspect_snapshot().nodes;

    // A computed `background-color` is always *present* (its initial value is
    // transparent), so the claim is about the value: the button's is the UA paper
    // fill — opaque — and the `<div>`'s is still the transparent initial one,
    // because no UA rule names `div`.
    let background = |class: &str| {
        let Some((_, value)) = node(&nodes, class)
            .styles
            .iter()
            .find(|(name, _)| name == "BackgroundColor")
        else {
            panic!("no background-color on .{class}")
        };
        value.clone()
    };
    let button_background = background("btn");
    assert!(
        button_background.contains("a: 1.0"),
        "the UA `button` rule must reach an authored button: {button_background}"
    );
    assert_ne!(
        button_background,
        background("plain"),
        "no UA rule names `div`, so the box beside it keeps the initial value"
    );
    // The label sits inside the UA padding + border, so the button is wider than
    // its text: a real box rather than a zero-sized one.
    assert!(
        width(&nodes, "btn") > 40.0,
        "the UA padding/border must be laid out: {:?}",
        node(&nodes, "btn").bounds
    );
}
