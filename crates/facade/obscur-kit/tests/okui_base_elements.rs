#![doc = include_str!("okui_base_elements.doc/_module.md")]
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
use obscur_kit::interaction::command::{KeyEvent, LogicalKey, NamedKey, PhysicalKey};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::okui_compiler::bridge::UiIrViewBridge;
use obscur_kit::okui_compiler::compile::Compiler;
use obscur_kit::okui_syntax::ast::Document;
use obscur_kit::okui_syntax::parser::Parser;
use obscur_kit::primitives::Container;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::orchestration::NodeInspection;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::style::property::PropertyId;
use obscur_kit::ui_ir::name_binding::{self, TagPosition};
use obscur_kit::ui_macros::okui;
use obscur_kit::view::view::View;
use obscur_kit::widgets::button::Button;
use obscur_kit::widgets::checkbox::Checkbox;
use obscur_kit::widgets::input::Input;
use obscur_kit::widgets::radio::Radio;
use obscur_kit::widgets::select::Select;
use obscur_kit::widgets::table::Table;

#[doc = include_str!("okui_base_elements.doc/stylesheet.md")]
fn stylesheet(css: &str) -> Stylesheet {
    let parsed = StylesheetParser::new(&CssSource::anonymous(css)).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("okui_base_elements.doc/render.md")]
fn render(view: &View, sheets: &[Stylesheet]) -> Vec<NodeInspection> {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 200.0));
    runtime.render_now(view, sheets);
    runtime.inspect_snapshot().nodes
}

#[doc = include_str!("okui_base_elements.doc/node.md")]
fn node<'a>(nodes: &'a [NodeInspection], class: &str) -> &'a NodeInspection {
    let mut found = nodes
        .iter()
        .filter(|node| node.classes.iter().any(|name| name == class));
    let node = found.next().unwrap_or_else(|| panic!("no .{class} node"));
    assert!(found.next().is_none(), "more than one .{class} node");
    node
}

#[doc = include_str!("okui_base_elements.doc/width.md")]
fn width(nodes: &[NodeInspection], class: &str) -> f32 {
    node(nodes, class).bounds.map_or(0.0, |r| r.size.width)
}

#[doc = include_str!("okui_base_elements.doc/interpreted.md")]
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
fn an_authored_type_attribute_and_the_tag_are_two_channels() {
    // THE defect this vocabulary work exists to close. `<button type="submit">` is
    // ordinary, idiomatic markup, and it used to be an element whose CSS *type* was
    // `submit`: it matched no `button { … }` rule, silently lost the whole of its UA
    // styling, and `[type=submit]` could not be written to mean the attribute
    // because the attribute lookup returned the local name. The tag says *what a
    // thing is*, `type` says *which kind*, and both now reach the cascade.
    let view = okui! {
        <div class="root">
            <button type="submit" class="btn">"OK"</button>
        </div>
    };
    // Still a `button` to a type selector.
    let nodes = render(&view, &[stylesheet("button { width: 120px }")]);
    assert_eq!(node(&nodes, "btn").local_name.as_deref(), Some("button"));
    assert_eq!(width(&nodes, "btn"), 120.0);

    // And the authored attribute is what `[type=submit]` reads.
    let by_attribute = render(&view, &[stylesheet("[type=submit] { width: 90px }")]);
    assert_eq!(width(&by_attribute, "btn"), 90.0);

    // Both at once, from one sheet: the type selector and the attribute selector
    // name the same node, and the more specific attribute rule wins the width.
    let both = render(
        &view,
        &[stylesheet(
            "button { width: 120px; height: 40px } [type=submit] { width: 90px }",
        )],
    );
    assert_eq!(width(&both, "btn"), 90.0);
    assert_eq!(node(&both, "btn").bounds.map(|r| r.size.height), Some(40.0));

    // The tag never answers for the attribute: a `<button>` with no authored `type`
    // matches no `[type=…]` rule at all.
    let plain = okui! { <div class="root"><button class="btn">"OK"</button></div> };
    let unmatched = render(&plain, &[stylesheet("[type=button] { width: 77px }")]);
    assert_ne!(width(&unmatched, "btn"), 77.0);
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

#[test]
fn every_control_the_ua_sheet_styles_is_the_shipped_widgets_id_in_markup() {
    // `<button>` was the only lowercase control tag; the rest named no built-in and
    // fell through to `Container`, so `<checkbox>` was a styled box with no
    // behaviour standing beside `Checkbox::new()`, the real control — one CSS type
    // over two different trees. Rendered rather than resolved, because the claim is
    // about the node that reaches the tree.
    let view = okui! {
        <div class="root">
            <input class="tf"/>
            <textarea class="ta"/>
            <checkbox class="cb"/>
            <radio class="rb"/>
            <select class="sel"/>
            <table class="tbl"/>
        </div>
    };
    let nodes = render(&view, &[]);
    for (class, local_name, component) in [
        ("tf", "input", Input::COMPONENT_ID),
        ("ta", "textarea", Input::COMPONENT_ID),
        ("cb", "checkbox", Checkbox::COMPONENT_ID),
        ("rb", "radio", Radio::COMPONENT_ID),
        ("sel", "select", Select::COMPONENT_ID),
        ("tbl", "table", Table::COMPONENT_ID),
    ] {
        let element = node(&nodes, class);
        assert_eq!(element.local_name.as_deref(), Some(local_name), "{class}");
        assert_eq!(element.component, Some(component), "{class}");
        assert_ne!(element.component, Some(Container::COMPONENT_ID), "{class}");
    }
}

#[test]
fn an_authored_textarea_is_the_input_widget_and_still_the_multiline_one() {
    // The interesting spelling, and it needed no new mechanism. HTML's two
    // text-entry tags are one widget here, so `<textarea>` binds to
    // `Input::COMPONENT_ID` exactly as `<input>` does — and the distinction they
    // carry survives in the LOCAL_NAME, which is what both the cascade and
    // `EditableField::is_multiline` read. So one id, and still two behaviours:
    // the UA sheet gives the single-line field `white-space: pre` (no wrapping,
    // no trailing-space hang) and the multi-line one `pre-wrap`.
    let view = okui! {
        <div class="root">
            <input class="tf"/>
            <textarea class="ta"/>
        </div>
    };
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 300.0));
    runtime.set_default_stylesheet(obscur_kit::ua_stylesheet());
    runtime.render_now(&view, &[]);
    let nodes = runtime.inspect_snapshot().nodes;

    let white_space = |class: &str| {
        runtime
            .computed_style(node(&nodes, class).node)
            .and_then(|style| style.get(PropertyId::WhiteSpace))
            .map_or_else(
                || panic!("no white-space on .{class}"),
                |value| format!("{value:?}"),
            )
    };
    assert_eq!(
        node(&nodes, "ta").component,
        node(&nodes, "tf").component,
        "one control, two tags"
    );
    assert_ne!(
        white_space("ta"),
        white_space("tf"),
        "the local name is what keeps `input` and `textarea` two behaviours: {:?} vs {:?}",
        white_space("ta"),
        white_space("tf")
    );
    // And the UA's intrinsic sizes, which are also written as type selectors: the
    // 180px/240px content widths of `input`/`textarea` plus the 13px padding and
    // 1.5px border the two share.
    assert_eq!(width(&nodes, "tf"), 180.0 + 26.0 + 3.0);
    assert_eq!(width(&nodes, "ta"), 240.0 + 26.0 + 3.0);
}

#[doc = include_str!("okui_base_elements.doc/typed_into.md")]
fn typed_into(class: &str) -> Option<String> {
    let view = okui! {
        <div class="root">
            <input class="tf" on:input={edit}/>
            <textarea class="ta" on:input={edit}/>
        </div>
    };
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(600.0, 400.0));
    runtime.set_default_stylesheet(obscur_kit::ua_stylesheet());
    runtime.on_text_input(InternedSymbol::named("edit"), |_| {});
    runtime.render_now(&view, &[]);

    // Press inside the field so the runtime seeds an editing session on it.
    let bounds = node(&runtime.inspect_snapshot().nodes, class)
        .bounds
        .expect("the UA sheet gives the field a box");
    let at = Point::new(bounds.origin.x + 5.0, bounds.origin.y + 5.0);
    for kind in [PointerEventKind::Down, PointerEventKind::Up] {
        let _ = runtime.handle_pointer(
            PointerEvent::new(PointerId::MOUSE, PointerDevice::Mouse, kind, at).as_trusted(),
        );
    }
    for key in [
        KeyEvent::press(PhysicalKey::KeyA, Some(LogicalKey::character('a'))),
        KeyEvent::press(PhysicalKey::Enter, Some(LogicalKey::named(NamedKey::Enter))),
        KeyEvent::press(PhysicalKey::KeyB, Some(LogicalKey::character('b'))),
    ] {
        runtime.handle_key(key);
    }
    runtime.editing_text().map(str::to_owned)
}

#[test]
fn an_authored_textareas_enter_inserts_a_newline_and_an_inputs_does_not() {
    // The behavioural half, and the reason `textarea` needed no second component
    // id: `EditableField::is_multiline` decides multi-line editing by
    // `local_name() == "textarea"`, and the tag is where the local name comes
    // from. So markup gets the distinction from the mechanism that was already
    // there — an `Enter` in the area is an edit, and in the field it is an
    // activation that inserts nothing.
    assert_eq!(typed_into("ta").as_deref(), Some("a\nb"));
    assert_eq!(typed_into("tf").as_deref(), Some("ab"));
}

#[test]
fn the_ua_stylesheet_reaches_an_authored_checkbox() {
    // `<checkbox>` used to be a `div`: it took the `checkbox { … }` rules (a local
    // name has always come from the tag) but not the control's identity, so markup
    // and `Checkbox::new()` were two nodes that merely looked alike. Both halves
    // are asserted here — the id, and the UA look that goes with it.
    let view = okui! {
        <div class="root">
            <checkbox class="cb"/>
            <div class="plain"/>
        </div>
    };
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 200.0));
    runtime.set_default_stylesheet(obscur_kit::ua_stylesheet());
    runtime.render_now(&view, &[]);
    let nodes = runtime.inspect_snapshot().nodes;

    assert_eq!(node(&nodes, "cb").component, Some(Checkbox::COMPONENT_ID));
    // The `checkbox { … }` rule sets no background (a bare type rule would box the
    // whole labelled control rather than the indicator); what it does set is the
    // ink and a flex row, so those are what the assertion reads.
    let computed = |class: &str, property: PropertyId| {
        runtime
            .computed_style(node(&nodes, class).node)
            .and_then(|style| style.get(property))
            .map(|value| format!("{value:?}"))
    };
    let ink = computed("cb", PropertyId::Color).expect("the UA rule declares a colour");
    assert!(
        ink.contains("0.16862746"),
        "the UA `checkbox` rule must reach an authored checkbox: {ink}"
    );
    assert!(
        computed("cb", PropertyId::Display).is_some(),
        "…and give it the indicator/label row the rule declares"
    );
    assert_eq!(
        computed("plain", PropertyId::Display),
        None,
        "no UA rule names `div`, so the box beside it declares nothing at all"
    );
    assert_ne!(
        computed("plain", PropertyId::Color),
        Some(ink),
        "…and keeps the initial ink, where the control's is the sheet's own"
    );
}
