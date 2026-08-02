#![doc = include_str!("builtin_components.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::bootstrap::{BuiltinComponents, KitBootstrap};
use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{InternedSymbol, RuntimeId};
use obscur_kit::interaction::command::{KeyEvent, LogicalKey, NamedKey, PhysicalKey};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::orchestration::NodeInspection;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::style::property::PropertyId;
use obscur_kit::ui_ir::name_binding;
use obscur_kit::ui_macros::{Contract, okui};
use obscur_kit::view::view::View;
use obscur_kit::widgets::button::{Button, ButtonProps};
use obscur_kit::widgets::checkbox::{Checkbox, CheckboxProps};
use obscur_kit::widgets::input::{Input, InputProps};
use obscur_kit::widgets::radio::{Radio, RadioProps};
use obscur_kit::widgets::select::{Select, SelectProps};

#[doc = include_str!("builtin_components.doc/MirrorCheckboxProps.md")]
#[derive(Contract)]
#[contract(name = "Checkbox")]
#[expect(
    dead_code,
    reason = "the derive's OUTPUT is the subject: this struct exists so the \
              generated contract can be compared with the hand-written one, and \
              nothing ever reads an instance of it"
)]
struct MirrorCheckboxProps {
    label: String,
    checked: Option<bool>,
}

#[doc = include_str!("builtin_components.doc/MirrorRadioProps.md")]
#[derive(Contract)]
#[contract(name = "Radio")]
#[expect(
    dead_code,
    reason = "the derive's OUTPUT is the subject: this struct exists so the \
              generated contract can be compared with the hand-written one, and \
              nothing ever reads an instance of it"
)]
struct MirrorRadioProps {
    label: String,
    #[contract(default, name = "checked")]
    selected: bool,
}

#[doc = include_str!("builtin_components.doc/MirrorSelectProps.md")]
#[derive(Contract)]
#[contract(name = "Select")]
#[expect(
    dead_code,
    reason = "the derive's OUTPUT is the subject: this struct exists so the \
              generated contract can be compared with the hand-written one, and \
              nothing ever reads an instance of it"
)]
struct MirrorSelectProps {
    options: Option<String>,
    selected: Option<usize>,
    #[contract(default)]
    open: bool,
}

#[doc = include_str!("builtin_components.doc/MirrorInputProps.md")]
#[derive(Contract)]
#[contract(name = "Input")]
#[expect(
    dead_code,
    reason = "the derive's OUTPUT is the subject: this struct exists so the \
              generated contract can be compared with the hand-written one, and \
              nothing ever reads an instance of it"
)]
struct MirrorInputProps {
    #[contract(default)]
    value: String,
}

#[doc = include_str!("builtin_components.doc/MirrorButtonProps.md")]
#[derive(Contract)]
#[contract(name = "Button", children = any)]
#[contract(slot(default))]
#[expect(
    dead_code,
    reason = "the derive's OUTPUT is the subject: this struct exists so the \
              generated contract can be compared with the hand-written one, and \
              nothing ever reads an instance of it"
)]
struct MirrorButtonProps {
    #[contract(default)]
    label: String,
}

#[doc = include_str!("builtin_components.doc/stylesheet.md")]
fn stylesheet(css: &str) -> Stylesheet {
    let parsed = StylesheetParser::new(&CssSource::anonymous(css)).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("builtin_components.doc/runtime.md")]
fn runtime(builtins: bool) -> UiRuntime {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 200.0));
    let boot = KitBootstrap::new();
    boot.install_default_stylesheet(&mut runtime);
    if builtins {
        boot.install_builtin_components(&mut runtime)
            .expect("the built-ins register on a fresh runtime");
    }
    runtime
}

#[doc = include_str!("builtin_components.doc/node.md")]
fn node<'a>(nodes: &'a [NodeInspection], class: &str) -> &'a NodeInspection {
    nodes
        .iter()
        .find(|node| node.classes.iter().any(|name| name == class))
        .unwrap_or_else(|| panic!("no .{class} node"))
}

#[doc = include_str!("builtin_components.doc/FAINT.md")]
const FAINT: (f32, f32, f32) = (
    0x9b as f32 / 255.0,
    0x99 as f32 / 255.0,
    0x8f as f32 / 255.0,
);

#[doc = include_str!("builtin_components.doc/painted_brush.md")]
fn painted_brush(runtime: &UiRuntime, text: &str) -> Option<(f32, f32, f32)> {
    runtime
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .find_map(|command| match command {
            PaintCommand::DrawText { run, brush, .. } if run.text() == text => brush
                .as_solid()
                .map(|solid| solid.color())
                .map(|c| (c.r, c.g, c.b)),
            _ => None,
        })
}

#[doc = include_str!("builtin_components.doc/typed_into.md")]
fn typed_into(class: &str) -> Option<String> {
    let view = okui! {
        <div class="root">
            <input class="tf" on:input={edit}/>
            <textarea class="ta" on:input={edit}/>
        </div>
    };
    let mut runtime = runtime(true);
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

#[doc = include_str!("builtin_components.doc/shape.md")]
fn shape(nodes: &[NodeInspection]) -> Vec<(u32, &'static str, Vec<String>, Option<String>)> {
    nodes
        .iter()
        .map(|node| {
            (
                node.depth,
                node.kind,
                node.classes.clone(),
                node.text.clone(),
            )
        })
        .collect()
}

// ---- 1. The ids and the contract line up ---------------------------------

#[test]
fn the_hand_written_contract_is_the_one_the_derive_would_emit() {
    // `#[derive(Contract)]` expands to `::obscur_kit::…` paths, and §3.16 forbids
    // an internal crate from depending on the facade — so a SHIPPED widget cannot
    // derive its own contract, and `CheckboxProps::contract()` is written out by
    // hand. A hand-mirrored value rots silently, so pin it here, in the one crate
    // that can hold both: same name in, same contract out, compared whole.
    assert_eq!(CheckboxProps::contract(), MirrorCheckboxProps::contract());
}

#[test]
fn the_derived_id_is_the_id_an_authored_checkbox_carries() {
    // Measured, not assumed. `contract_derive::component_id` consults
    // `builtin_component_id` first, so the name "Checkbox" resolves to the
    // reserved constant rather than to a hash of the name — which is what makes a
    // factory registered under this contract reachable from `<checkbox>` markup.
    assert_eq!(MirrorCheckboxProps::contract().id, name_binding::CHECKBOX);
    assert_eq!(CheckboxProps::contract().id, name_binding::CHECKBOX);
    assert_eq!(Checkbox::COMPONENT_ID, name_binding::CHECKBOX);
}

#[test]
fn the_label_prop_symbol_is_the_one_authored_markup_carries() {
    // The components tier restates this symbol because it may not depend on the
    // `name_binding` registry that computes it. Same drift risk, same remedy.
    assert_eq!(CheckboxProps::LABEL, name_binding::attr_symbol("label"));
    // One attribute name, one symbol: a button's label and a checkbox's are the
    // same authored `label="…"`, and two spellings of it would mean an authored
    // value that reaches one control and not the other.
    assert_eq!(ButtonProps::LABEL, name_binding::attr_symbol("label"));
}

#[test]
fn the_derived_id_is_the_id_an_authored_button_carries() {
    assert_eq!(MirrorButtonProps::contract().id, name_binding::BUTTON);
    assert_eq!(ButtonProps::contract().id, name_binding::BUTTON);
    assert_eq!(Button::COMPONENT_ID, name_binding::BUTTON);
}

#[test]
fn the_buttons_contract_is_the_derives_output_whole_default_slot_included() {
    // `ButtonProps::contract()` is hand-written for the same §3.16 reason as the
    // others: the derive expands to `::obscur_kit::…` paths and an internal
    // crate may not depend on the facade.
    //
    // This assertion used to stop short of the slot, under the name
    // "…plus_a_slot_no_derive_can_spell": `#[contract(slot(name = "…"))]` names
    // a slot and this control's is the unnamed one, so the derive had no way to
    // say it. It does now — §2.12 states the default slot and `slot(default)`
    // is its spelling — so the two contracts are compared WHOLE, which is what
    // the other four already were.
    let hand = ButtonProps::contract();
    let derived = MirrorButtonProps::contract();
    assert_eq!(hand, derived);
    assert_eq!(hand.slots.len(), 1);
    assert_eq!(hand.slots[0].name, obscur_kit::view::view::DEFAULT_SLOT);
}

// ---- 2. An authored checkbox mounts the real control ----------------------

#[test]
fn an_authored_checkbox_renders_the_control_not_an_empty_element() {
    // THE acceptance test for this stage. Before, `<checkbox>` was an element with
    // the right id and nothing inside it: "the id is a handle with nothing mounted
    // under it". Now the widget's own `view()` runs.
    let view = okui! {
        <div class="root">
            <checkbox class="cb" label="Agree" checked/>
        </div>
    };
    let mut runtime = runtime(true);
    runtime.render_now(&view, &[]);
    let nodes = runtime.inspect_snapshot().nodes;

    // The host node is untouched: same id, same local name, still the cascade's
    // `checkbox` element.
    let host = node(&nodes, "cb");
    assert_eq!(host.component, Some(Checkbox::COMPONENT_ID));
    assert_eq!(host.local_name.as_deref(), Some("checkbox"));

    // …and under it, the control the widget builds.
    let indicator = node(&nodes, "checkbox-box");
    assert_eq!(indicator.parent, Some(host.node));
    assert!(
        indicator.classes.iter().any(|c| c == "checked"),
        "a checked checkbox's indicator carries the state class: {:?}",
        indicator.classes
    );
    assert!(
        nodes.iter().any(|n| n.text.as_deref() == Some("Agree")),
        "the label prop is rendered as the control's text"
    );
}

#[test]
fn without_the_registration_the_same_markup_is_still_an_empty_element() {
    // The other half of the claim above: the contents come from the registration
    // and from nothing else, so the change is attributable.
    let view = okui! {
        <div class="root">
            <checkbox class="cb" label="Agree" checked/>
        </div>
    };
    let mut runtime = runtime(false);
    runtime.render_now(&view, &[]);
    let nodes = runtime.inspect_snapshot().nodes;

    assert_eq!(node(&nodes, "cb").component, Some(Checkbox::COMPONENT_ID));
    assert!(
        !nodes
            .iter()
            .any(|n| n.classes.iter().any(|c| c == "checkbox-box")),
        "nothing mounts without a registered factory"
    );
}

#[test]
fn the_ua_sheet_fills_the_mounted_indicator_and_checked_still_reaches_the_cascade() {
    // Two cascade claims on one tree, because they are the two halves that used to
    // be impossible to have at once: the sheet's `.checkbox-box.checked` rule had
    // no node to match (the sub-part never existed in markup), and `:checked` on
    // the host had to keep working after the component pass rewrote the children.
    let view = okui! {
        <div class="root">
            <checkbox class="cb" label="Agree" checked/>
        </div>
    };
    let author = stylesheet("checkbox:checked { color: #ffffff; }");
    let mut runtime = runtime(true);
    runtime.render_now(&view, &[author]);
    let nodes = runtime.inspect_snapshot().nodes;

    let computed = |target: &NodeInspection, property: PropertyId| {
        runtime
            .computed_style(target.node)
            .and_then(|style| style.get(property))
            .map(|value| format!("{value:?}"))
    };

    // The UA `.checkbox-box.checked` fill (#2b2a27) reaches the mounted sub-part.
    // A component's output is style-SCOPED by default (§4.15), and this is why
    // that is not a problem: the built-in definition of the base elements is at
    // the SystemDefault origin, which the scoping rule deliberately lets through.
    let ink = computed(node(&nodes, "checkbox-box"), PropertyId::BackgroundColor)
        .expect("the UA rule fills the checked indicator");
    assert!(
        ink.contains("0.16862746"),
        "the UA `.checkbox-box.checked` fill must reach the mounted indicator: {ink}"
    );

    // And the author's `checkbox:checked` still matches the host, so mounting did
    // not cost the control its pseudo-class.
    let color =
        computed(node(&nodes, "cb"), PropertyId::Color).expect("the author rule declares a colour");
    assert!(
        color.contains("1.0"),
        "`checkbox:checked` must still match the mounted host: {color}"
    );
}

// ---- 3. Mounting is idempotent on the Rust API ---------------------------

#[test]
fn a_built_checkbox_renders_the_same_tree_whether_or_not_the_built_ins_are_registered() {
    // The trap this stage had to close: `Checkbox::new(…).view()` carries the SAME
    // component id as `<checkbox>`, so registering a factory mounts a component on
    // the Rust API's own node too. If the component could not reproduce that
    // node's children from its props, every existing program's checkboxes would
    // lose their labels the day the built-ins were registered.
    //
    // It can, because `view()` now emits `label` and the tri-state as PROPS, and
    // the component renders `Checkbox::contents` from them — the same body `view()`
    // installs.
    let command = InternedSymbol::from_raw(0x1234);
    for built in [
        Checkbox::new("Agree", command).checked(true),
        Checkbox::new("Agree", command).checked(false),
        Checkbox::new("Agree", command).disabled(true),
        Checkbox::indeterminate("Agree", command),
    ] {
        let view: View = built.view();
        let mut bare = runtime(false);
        bare.render_now(&view, &[]);
        let mut mounted = runtime(true);
        mounted.render_now(&view, &[]);
        assert_eq!(
            shape(&bare.inspect_snapshot().nodes),
            shape(&mounted.inspect_snapshot().nodes),
            "mounting the built-in must not change what {built:?} renders"
        );
    }
}

#[test]
fn a_labelless_authored_checkbox_reports_rather_than_mounting_a_blank_control() {
    // `label` is required, mirroring the §2.13 surface `widget_contracts()` already
    // ships (`requiring_prop("label")`). The observable consequence, stated rather
    // than discovered: the node stays, nothing is rendered under it, and the error
    // boundary has a props failure to report — instead of a silently blank control.
    let view = okui! {
        <div class="root">
            <checkbox class="cb"/>
        </div>
    };
    let mut runtime = runtime(true);
    runtime.render_now(&view, &[]);
    let nodes = runtime.inspect_snapshot().nodes;

    assert_eq!(node(&nodes, "cb").component, Some(Checkbox::COMPONENT_ID));
    assert!(
        !nodes
            .iter()
            .any(|n| n.classes.iter().any(|c| c == "checkbox-box")),
        "a checkbox whose props do not decode renders nothing, not a half-control"
    );
}

// ---- 4. Radio: the same shape, and the spelling it forced ----------------

#[test]
fn the_hand_written_radio_contract_is_the_one_the_derive_would_emit() {
    assert_eq!(RadioProps::contract(), MirrorRadioProps::contract());
    assert_eq!(MirrorRadioProps::contract().id, name_binding::RADIO);
    assert_eq!(Radio::COMPONENT_ID, name_binding::RADIO);
    // The components tier restates this symbol because it may not depend on the
    // `name_binding` registry that computes it. Same drift risk, same remedy.
    assert_eq!(RadioProps::LABEL, name_binding::attr_symbol("label"));
    // And the selection's authored name is the RESERVED attribute — the one the
    // cascade reads for `:checked` — not the builder's method name.
    assert_eq!(
        RadioProps::contract().props[1].name,
        name_binding::attr_symbol("checked")
    );
    assert_ne!(
        RadioProps::contract().props[1].name,
        name_binding::attr_symbol("selected")
    );
}

#[test]
fn an_authored_radio_renders_the_dot_and_its_checked_state_reaches_the_cascade() {
    // Both halves at once, as for the checkbox: the sub-part the UA sheet fills
    // never existed in markup before, and `:checked` on the host must survive the
    // component pass rewriting the children.
    let view = okui! {
        <div class="root">
            <radio class="rb" label="One" checked/>
        </div>
    };
    let author = stylesheet("radio:checked { color: #ffffff; }");
    let mut runtime = runtime(true);
    runtime.render_now(&view, &[author]);
    let nodes = runtime.inspect_snapshot().nodes;

    let host = node(&nodes, "rb");
    assert_eq!(host.component, Some(Radio::COMPONENT_ID));
    assert_eq!(host.local_name.as_deref(), Some("radio"));

    let dot = node(&nodes, "radio-dot");
    assert_eq!(dot.parent, Some(host.node));
    assert!(
        nodes
            .iter()
            .any(|n| n.classes.iter().any(|c| c == "radio-fill")),
        "a checked radio paints its filled centre: {:?}",
        nodes.iter().map(|n| &n.classes).collect::<Vec<_>>()
    );
    assert!(
        nodes.iter().any(|n| n.text.as_deref() == Some("One")),
        "the label prop is rendered as the control's text"
    );

    // The painted dot and the matched `:checked` are the same fact, so they cannot
    // disagree — which is why `checked` is the authored spelling and `selected`
    // (the builder's method name) is not.
    let colour = runtime
        .computed_style(host.node)
        .and_then(|style| style.get(PropertyId::Color))
        .map(|value| format!("{value:?}"))
        .expect("the author rule declares a colour");
    assert!(
        colour.contains("1.0"),
        "`radio:checked` must still match the mounted host: {colour}"
    );
}

#[test]
fn a_built_radio_renders_the_same_tree_whether_or_not_the_built_ins_are_registered() {
    let command = InternedSymbol::from_raw(0x5678);
    for built in [
        Radio::new("One", command),
        Radio::new("One", command).selected(true),
        Radio::new("One", command).disabled(true),
    ] {
        let view: View = built.view();
        let mut bare = runtime(false);
        bare.render_now(&view, &[]);
        let mut mounted = runtime(true);
        mounted.render_now(&view, &[]);
        assert_eq!(
            shape(&bare.inspect_snapshot().nodes),
            shape(&mounted.inspect_snapshot().nodes),
            "mounting the built-in must not change what {built:?} renders"
        );
    }
}

// ---- 5. Select: two ways to state options, one tree ----------------------

#[test]
fn the_hand_written_select_contract_is_the_one_the_derive_would_emit() {
    assert_eq!(SelectProps::contract(), MirrorSelectProps::contract());
    assert_eq!(MirrorSelectProps::contract().id, name_binding::SELECT);
    assert_eq!(Select::COMPONENT_ID, name_binding::SELECT);
    assert_eq!(SelectProps::OPTIONS, name_binding::attr_symbol("options"));
    assert_eq!(SelectProps::SELECTED, name_binding::attr_symbol("selected"));
    assert_eq!(SelectProps::OPEN, name_binding::attr_symbol("open"));
}

#[test]
fn an_authored_select_and_a_built_one_render_the_same_tree() {
    // THE acceptance test for this stage's one real decision. A select's options
    // are labels, not views, so markup states them as a comma-separated PROP
    // rather than as projected children — and the two front ends must therefore
    // agree, node for node, on what that produces. Anything less and `<select>`
    // would be a second, subtly different select standing beside the Rust one,
    // which is the exact fault this whole campaign is closing.
    let authored: View = okui! { <select options="Alpha, Beta" selected={1} open/> };
    let mut built = Select::new(InternedSymbol::from_raw(0x9ABC)).with_options(["Alpha", "Beta"]);
    assert!(built.select(1), "the second option must be selectable");
    built.open(); // `select` closes the popup, so it is reopened after the pick.

    let mut from_markup = runtime(true);
    from_markup.render_now(&authored, &[]);
    let mut from_rust = runtime(true);
    from_rust.render_now(&built.view(), &[]);
    assert_eq!(
        shape(&from_markup.inspect_snapshot().nodes),
        shape(&from_rust.inspect_snapshot().nodes)
    );

    // And it is a real control, not two matching empties.
    let nodes = from_markup.inspect_snapshot().nodes;
    assert_eq!(node(&nodes, "select-list").classes, vec!["select-list"]);
    assert!(
        nodes
            .iter()
            .any(|n| n.classes.iter().any(|c| c == "select-option")
                && n.classes.iter().any(|c| c == "selected")),
        "the picked row carries the `selected` class"
    );
    assert!(
        nodes.iter().any(|n| n.text.as_deref() == Some("Beta")),
        "the picked label is the header's"
    );
}

#[test]
fn a_disabled_selects_option_rows_still_read_muted_once_the_control_is_mounted() {
    // The one state the UA sheet's `select:disabled .select-option` rule can
    // match, and the one this stage could most easily have deleted: `Select::open`
    // refuses on a disabled control, so re-applying that refusal when RENDERING an
    // already-open disabled control would have left the rule with nothing to
    // match, silently. `SelectComponent` draws the popup from `open` alone.
    let view = okui! { <select options="Alpha, Beta" open disabled/> };
    let mut runtime = runtime(true);
    runtime.render_now(&view, &[]);
    let nodes = runtime.inspect_snapshot().nodes;

    let row = node(&nodes, "select-option");
    let colour = runtime
        .computed_style(row.node)
        .and_then(|style| style.get(PropertyId::Color))
        .map(|value| format!("{value:?}"))
        .expect("an option row has a computed colour");
    assert!(
        colour.contains("0.7176"),
        "`select:disabled .select-option` must reach the mounted rows (#b7b5ad): {colour}"
    );
}

#[test]
fn a_built_select_renders_the_same_tree_whether_or_not_the_built_ins_are_registered() {
    let command = InternedSymbol::from_raw(0x9ABC);
    let mut open = Select::new(command).with_options(["Alpha", "Beta"]);
    open.open();
    let mut disabled_open = Select::new(command).with_options(["Alpha", "Beta"]);
    disabled_open.open();
    disabled_open.set_disabled(true);
    // A label a comma-separated attribute could never carry, so the widget's own
    // lossless spelling is exercised rather than assumed.
    let comma = Select::new(command).with_options(["a,b"]);

    for built in [
        Select::new(command),
        Select::new(command).with_options(["Alpha"]),
        open,
        disabled_open,
        comma,
    ] {
        let view: View = built.view();
        let mut bare = runtime(false);
        bare.render_now(&view, &[]);
        let mut mounted = runtime(true);
        mounted.render_now(&view, &[]);
        assert_eq!(
            shape(&bare.inspect_snapshot().nodes),
            shape(&mounted.inspect_snapshot().nodes),
            "mounting the built-in must not change what {built:?} renders"
        );
    }
}

// ---- 6. Input: two tags, one component, and a hint that is not in it -----

#[test]
fn the_hand_written_input_contract_is_the_one_the_derive_would_emit() {
    assert_eq!(InputProps::contract(), MirrorInputProps::contract());
    assert_eq!(MirrorInputProps::contract().id, name_binding::INPUT);
    assert_eq!(Input::COMPONENT_ID, name_binding::INPUT);
    assert_eq!(InputProps::VALUE, name_binding::attr_symbol("value"));
    // And the symbol the value travels on is NOT the reserved `VALUE`: that one
    // declares a *controlled* value the runtime snaps a drifting buffer back to,
    // while markup's `value="…"` is HTML's default value, which the user then
    // owns. One symbol for both would make every authored field uneditable.
    assert_ne!(InputProps::VALUE, obscur_kit::view::view::VALUE);
}

#[test]
fn an_authored_input_renders_its_value_and_without_the_registration_does_not() {
    // The acceptance test for this stage, in both directions at once. `<input>`
    // used to be an element with the right id and nothing inside it, so an
    // authored value was invisible; now the widget's own body runs. The second
    // half is what makes the change attributable to the registration and to
    // nothing else.
    let view = okui! {
        <div class="root">
            <input class="tf" value="abc"/>
        </div>
    };
    let mut mounted = runtime(true);
    mounted.render_now(&view, &[]);
    let nodes = mounted.inspect_snapshot().nodes;

    // The host node is untouched: same id, same local name, still the cascade's
    // `input` element.
    let host = node(&nodes, "tf");
    assert_eq!(host.component, Some(Input::COMPONENT_ID));
    assert_eq!(host.local_name.as_deref(), Some("input"));
    assert!(
        nodes
            .iter()
            .any(|n| n.parent == Some(host.node) && n.text.as_deref() == Some("abc")),
        "the value prop is rendered as the field's editable run: {:?}",
        nodes.iter().map(|n| &n.text).collect::<Vec<_>>()
    );

    let mut bare = runtime(false);
    bare.render_now(&view, &[]);
    assert!(
        !bare
            .inspect_snapshot()
            .nodes
            .iter()
            .any(|n| n.text.is_some()),
        "nothing renders the value without a registered factory"
    );
}

#[test]
fn an_authored_inputs_placeholder_reaches_the_synthetic_part_at_last() {
    // THE prize attached to this stage, and it is worth being precise about what
    // paid for it: **mounting did not**. A component renders its host's CHILDREN,
    // and §4.16 makes the placeholder a synthetic part the runtime paints from a
    // reserved attribute on the host — never a child, and so never anything a
    // component could produce. What the hint was missing was the attribute: the
    // front ends hashed `placeholder="…"` into an ordinary prop that no part of
    // the pipeline reads, so `StyleAttributes::placeholder` saw nothing and no
    // part was generated. `name_binding::attr_symbol` now maps it to the reserved
    // symbol, which is the whole of the fix.
    assert_eq!(
        name_binding::attr_symbol("placeholder"),
        obscur_kit::view::view::PLACEHOLDER
    );

    let view = okui! {
        <div class="root">
            <input class="tf" placeholder="Search"/>
        </div>
    };
    let mut runtime = runtime(true);
    runtime.render_now(&view, &[]);
    let nodes = runtime.inspect_snapshot().nodes;

    let host = node(&nodes, "tf");
    let part = nodes
        .iter()
        .find(|n| n.kind == "synthetic" && n.local_name.as_deref() == Some("placeholder"))
        .expect("an authored placeholder generates the §4.16 synthetic part");
    assert_eq!(part.parent, Some(host.node), "attributed to its field");

    // …and it paints as a hint rather than as a value: the UA sheet's
    // `input::part(placeholder)` rule reaches it, in #9b998f rather than the
    // field's full #2b2a27 ink. A `::part()` subject is the ONLY selector that
    // can reach this part, which is why the paint is asserted and not just the
    // node's existence.
    assert_eq!(painted_brush(&runtime, "Search"), Some(FAINT));
}

#[test]
fn an_authored_value_hides_the_placeholder_the_way_a_declared_one_does() {
    // The defect that came in the same door as the prize. `:placeholder-shown`
    // asks whether the field's value is empty, and the runtime used to ask that
    // of the reserved `VALUE` attribute alone — which authored markup does not
    // write, on purpose (it declares a CONTROLLED value, and an authored
    // `value="…"` is HTML's default value the user then owns). So the moment the
    // hint became reachable from markup, `<input value="abc" placeholder="…"/>`
    // painted the hint on top of its own text: two values in one field.
    //
    // The rule is now "empty" as the field RENDERS it — the mounted control's
    // editable run — whenever no controlled value is declared.
    let view = okui! {
        <div class="root">
            <input class="tf" value="abc" placeholder="Search"/>
        </div>
    };
    let mut filled = runtime(true);
    filled.render_now(&view, &[]);
    let nodes = filled.inspect_snapshot().nodes;
    assert!(
        !nodes
            .iter()
            .any(|n| n.kind == "synthetic" && n.local_name.as_deref() == Some("placeholder")),
        "a field with a value must generate no placeholder part: {:?}",
        nodes.iter().map(|n| (n.kind, &n.text)).collect::<Vec<_>>()
    );
    assert_eq!(
        painted_brush(&filled, "Search"),
        None,
        "the hint is unpainted"
    );
    // …and it is the value that is on screen, so this is not the hint being lost
    // together with the text.
    assert!(
        painted_brush(&filled, "abc").is_some(),
        "the value is painted"
    );

    // The other half of the same rule, kept next to it: an EMPTY authored value
    // still shows the hint, so the rule reads the value rather than the mere
    // presence of the attribute.
    let bare = okui! {
        <div class="root">
            <input class="tf" value="" placeholder="Search"/>
        </div>
    };
    let mut empty = runtime(true);
    empty.render_now(&bare, &[]);
    assert_eq!(painted_brush(&empty, "Search"), Some(FAINT));
}

#[test]
fn mounting_leaves_a_textarea_multiline_and_an_input_single_line() {
    // The distinction the two tags carry is not in the component: `<input>` and
    // `<textarea>` share one id, and `EditableField::is_multiline` reads the
    // element's LOCAL NAME, which lives on the host node. A component render
    // replaces its host's children and never its props — so this passes because
    // mounting left the host alone, and it is here to fail loudly if that ever
    // stops being true.
    //
    // The editing session, the caret and the value round-trip are the runtime's,
    // driven against the retained tree; typing is the only way to ask whether
    // they still work through a mounted control.
    assert_eq!(typed_into("ta").as_deref(), Some("a\nb"));
    assert_eq!(typed_into("tf").as_deref(), Some("ab"));
}

#[test]
fn a_built_input_renders_the_same_tree_whether_or_not_the_built_ins_are_registered() {
    // The trap every stage of this campaign has to close: `Input::new(…).view()`
    // carries the same id as `<input>`, so registering a factory mounts a
    // component on the Rust API's own node too. If the component could not
    // reproduce that node's children from its props, every existing program's
    // fields would lose their text the day the built-ins were registered.
    let command = InternedSymbol::from_raw(0x5678);
    for built in [
        Input::new(command),
        Input::with_value(command, "abc"),
        Input::new(command).placeholder("Search"),
        Input::with_value(command, "abc").placeholder("Search"),
        Input::with_value(command, "line one\nline two").multiline(true),
        Input::with_value(command, "abc").readonly(true),
        Input::with_value(command, "abc").disabled(true),
    ] {
        let view: View = built.view();
        let mut bare = runtime(false);
        bare.render_now(&view, &[]);
        let mut mounted = runtime(true);
        mounted.render_now(&view, &[]);
        assert_eq!(
            shape(&bare.inspect_snapshot().nodes),
            shape(&mounted.inspect_snapshot().nodes),
            "mounting the built-in must not change what {built:?} renders"
        );
    }
}

// ---- The line no stage of this campaign may cross ------------------------

#[test]
fn an_authored_buttons_children_still_render() {
    // The campaign's acceptance test, kept here rather than assumed: a component
    // renders its own view, so mounting one on `<button>` would throw the
    // authored children away and `<button>Save</button>` would come out empty.
    // A factory IS registered for `BUTTON` now, and the reason this still holds
    // is the whole of the stage that registered it — `ButtonComponent` renders
    // the §2.12 default-slot outlet, into which the runtime projects exactly
    // these children before the render is reconciled.
    let view = okui! {
        <div class="root">
            <button class="btn">"Save"</button>
        </div>
    };
    let mut runtime = runtime(true);
    runtime.render_now(&view, &[]);
    let nodes = runtime.inspect_snapshot().nodes;
    assert!(
        nodes.iter().any(|n| n.text.as_deref() == Some("Save")),
        "`<button>Save</button>` must render its authored label: {:?}",
        nodes.iter().map(|n| &n.text).collect::<Vec<_>>()
    );
}

#[test]
fn an_authored_buttons_element_children_still_render_too() {
    // The same line, one step harder: `<button><span>press</span></button>` is
    // the `facade-consumer` fixture's markup and the spelling `okui_child_contracts`
    // blesses ("an authored `<button>`'s children are the only thing that
    // renders"). An element child is projected by the same rule as a text run —
    // naming no slot is what puts content in the default one.
    let view = okui! {
        <div class="root">
            <button class="btn"><span class="inner">"press"</span></button>
        </div>
    };
    let mut runtime = runtime(true);
    runtime.render_now(&view, &[]);
    let nodes = runtime.inspect_snapshot().nodes;
    let host = node(&nodes, "btn");
    let inner = node(&nodes, "inner");
    assert_eq!(
        inner.parent,
        Some(host.node),
        "the authored span stays the button's own child"
    );
    assert!(nodes.iter().any(|n| n.text.as_deref() == Some("press")));
}

#[test]
fn the_label_prop_spelling_renders_the_same_button() {
    // The half that did NOT work before this stage, and the reason the id was a
    // "handle with nothing mounted under it": the `.okui` compiler *requires*
    // `label` on `<Button/>`, and the result was an empty button — the toolkit
    // asking for a value nothing read. It is the outlet's fallback now.
    let view = okui! {
        <div class="root">
            <button class="btn" label="Save"/>
        </div>
    };
    let mut mounted = runtime(true);
    mounted.render_now(&view, &[]);
    let nodes = mounted.inspect_snapshot().nodes;
    let host = node(&nodes, "btn");
    assert_eq!(host.component, Some(Button::COMPONENT_ID));
    assert_eq!(host.local_name.as_deref(), Some("button"));
    assert!(
        nodes
            .iter()
            .any(|n| n.text.as_deref() == Some("Save") && n.parent == Some(host.node)),
        "`<button label=\"Save\"/>` renders the label as the button's own text: {:?}",
        nodes.iter().map(|n| &n.text).collect::<Vec<_>>()
    );

    // …and without the registration it renders nothing at all, so the change is
    // attributable to the factory and to nothing else.
    let mut bare = runtime(false);
    bare.render_now(&view, &[]);
    assert!(
        !bare
            .inspect_snapshot()
            .nodes
            .iter()
            .any(|n| n.text.as_deref() == Some("Save"))
    );
}

#[test]
fn authored_content_beats_the_label_prop_rather_than_displacing_it() {
    // Both spellings on one node is a confused document, and the rule for it is
    // the one §2.12 fallback already implies: the author's content is never
    // displaced, so nothing is lost silently. (The label is not rendered twice
    // either — the fallback is what an EMPTY slot shows.)
    let view = okui! {
        <div class="root">
            <button class="btn" label="Ignored">"Save"</button>
        </div>
    };
    let mut runtime = runtime(true);
    runtime.render_now(&view, &[]);
    let texts: Vec<String> = runtime
        .inspect_snapshot()
        .nodes
        .iter()
        .filter_map(|n| n.text.clone())
        .collect();
    assert!(texts.iter().any(|t| t == "Save"));
    assert!(
        !texts.iter().any(|t| t == "Ignored"),
        "the fallback belongs to the empty slot only: {texts:?}"
    );
}

#[test]
fn a_built_button_renders_the_same_tree_whether_or_not_the_built_ins_are_registered() {
    // The idempotence trap, for the control where it bites hardest: `Button::view`
    // states its label as a CHILD, not as a prop, so a component that rendered
    // from props alone would blank every button any existing program ever built.
    // It renders the outlet instead, and the child is projected back into it.
    let command = InternedSymbol::from_raw(0xABCD);
    for built in [
        Button::new("Save", command),
        Button::new("Save", command).disabled(true),
        Button::new("", command),
    ] {
        let view: View = built.view();
        let mut bare = runtime(false);
        bare.render_now(&view, &[]);
        let mut mounted = runtime(true);
        mounted.render_now(&view, &[]);
        assert_eq!(
            shape(&bare.inspect_snapshot().nodes),
            shape(&mounted.inspect_snapshot().nodes),
            "mounting the built-in must not change what {built:?} renders"
        );
    }
}

// ---- The registration set itself -----------------------------------------

#[test]
fn a_host_can_see_and_refuse_the_default_registration() {
    // The opt-out has to be visible on the value, not buried in a runner: a host
    // that ships its own `<checkbox>` must be able to stop the default from
    // claiming the id first.
    assert!(KitBootstrap::new().registers_builtin_components());
    let opted_out = KitBootstrap::new().without_builtin_components();
    assert!(!opted_out.registers_builtin_components());

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 200.0));
    opted_out
        .install_builtin_components(&mut runtime)
        .expect("opting out installs nothing, successfully");
    // Proof it installed nothing: the id is still free.
    BuiltinComponents::install(&mut runtime).expect("the built-in id was never claimed");
}
