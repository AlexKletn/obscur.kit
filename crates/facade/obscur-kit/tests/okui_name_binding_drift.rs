#![doc = include_str!("okui_name_binding_drift.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::ui_ir::name_binding::{self, TagPosition};
use obscur_kit::view::view;

use obscur_kit::primitives::{Canvas, Container, ForeignSurface, Image, Layer, Portal, Scroll};
use obscur_kit::widgets::button::Button;
use obscur_kit::widgets::checkbox::Checkbox;
use obscur_kit::widgets::dialog::Dialog;
use obscur_kit::widgets::docking::Docking;
use obscur_kit::widgets::input::Input;
use obscur_kit::widgets::list::List;
use obscur_kit::widgets::menu::Menu;
use obscur_kit::widgets::radio::Radio;
use obscur_kit::widgets::select::Select;
use obscur_kit::widgets::table::Table;
use obscur_kit::widgets::tabs::Tabs;
use obscur_kit::widgets::tree::Tree;

#[test]
fn reserved_attribute_symbols_match_obscur_view() {
    assert_eq!(name_binding::CLASS, view::CLASS);
    assert_eq!(name_binding::ID, view::ID);
    assert_eq!(name_binding::TYPE, view::TYPE);
    // The element's own name, carved out of TYPE so an authored `type="…"` and the
    // tag stop overwriting each other. Drift here un-styles the whole element
    // vocabulary on one front end and not the other.
    assert_eq!(name_binding::LOCAL_NAME, view::LOCAL_NAME);
    assert_ne!(name_binding::LOCAL_NAME, name_binding::TYPE);
    // §4.14 origin 5. Drift here means an authored `style="…"` lowers under a
    // symbol the style tier never reads: carried, and silently never applied.
    assert_eq!(name_binding::STYLE, view::STYLE);
    assert_eq!(name_binding::SRC, view::SRC);
    assert_eq!(name_binding::DISABLED, view::DISABLED);
    assert_eq!(name_binding::CHECKED, view::CHECKED);
    assert_eq!(name_binding::TEST_ID, view::TEST_ID);
    assert_eq!(name_binding::AUTOMATION_ID, view::AUTOMATION_ID);
    // §2.12: the front ends lower `slot="…"` under this symbol and the runtime
    // routes on it. Drift here silently un-routes every slot in the toolkit.
    assert_eq!(name_binding::SLOT, view::SLOT);
}

#[test]
fn the_slot_outlet_id_matches_obscur_view() {
    // §2.12: the front ends bind `<slot>` to this id and `project_slots` — which
    // lives below them and cannot call `name_binding` — is what recognises it.
    // Drift here makes every authored outlet an unregistered component that
    // mounts nothing, and the only symptom is a slot that never fills.
    assert_eq!(name_binding::SLOT_OUTLET, view::SLOT_OUTLET);
}

#[test]
fn the_authored_outlet_tag_binds_to_the_id_the_runtime_projects() {
    // The full authored spelling, pinned end to end across the two crates: the
    // tag resolves to the id `project_slots` substitutes, and `name` on that tag
    // resolves to the symbol it reads the outlet's slot from. Either half alone
    // is an outlet that never fills.
    assert_eq!(
        name_binding::component_id_for(name_binding::OUTLET_TAG, TagPosition::Element),
        view::SLOT_OUTLET
    );
    assert_eq!(
        name_binding::attr_symbol_on(name_binding::OUTLET_TAG, "name"),
        view::SLOT
    );
    // And the outlet a component builds by hand is the same node the tag lowers
    // to — one outlet, two spellings, not two outlets.
    assert!(view::is_outlet(&view::ViewNode::element(
        name_binding::component_id_for(name_binding::OUTLET_TAG, TagPosition::Element)
    )));
}

#[test]
fn reserved_event_symbols_match_obscur_view() {
    assert_eq!(name_binding::EVENT_CLICK, view::EVENT_CLICK);
    assert_eq!(name_binding::EVENT_INPUT, view::EVENT_INPUT);
    assert_eq!(name_binding::EVENT_CHANGE, view::EVENT_CHANGE);
    assert_eq!(name_binding::EVENT_LONG_PRESS, view::EVENT_LONG_PRESS);
    // And the markup spelling reaches it: `on:longpress="showMenu"` must lower to
    // the symbol the runtime fires a recognized hold on, or the binding silently
    // becomes an ordinary hashed name that nothing ever dispatches.
    assert_eq!(
        name_binding::event_symbol("longpress"),
        view::EVENT_LONG_PRESS
    );
    assert_eq!(name_binding::EVENT_KEY_DOWN, view::EVENT_KEY_DOWN);
    assert_eq!(name_binding::EVENT_KEY_UP, view::EVENT_KEY_UP);
    assert_eq!(name_binding::EVENT_POINTER_DOWN, view::EVENT_POINTER_DOWN);
    assert_eq!(name_binding::EVENT_POINTER_UP, view::EVENT_POINTER_UP);
    assert_eq!(name_binding::EVENT_POINTER_MOVE, view::EVENT_POINTER_MOVE);
    assert_eq!(
        name_binding::EVENT_POINTER_CANCEL,
        view::EVENT_POINTER_CANCEL
    );
    // The crossings, which are emitted from the hover transition rather than
    // from a `PointerEventKind` — so drift here is invisible on the runtime side
    // and shows up only as a hover interaction that never runs.
    assert_eq!(name_binding::EVENT_POINTER_ENTER, view::EVENT_POINTER_ENTER);
    assert_eq!(name_binding::EVENT_POINTER_LEAVE, view::EVENT_POINTER_LEAVE);
    // The wheel, which is neither a pointer kind nor a key: it is dispatched
    // from `handle_scroll`, so drift here is invisible on both other paths and
    // shows up only as a scroll nobody can refuse.
    assert_eq!(name_binding::EVENT_WHEEL, view::EVENT_WHEEL);
    // The focus pair, which is emitted from neither a pointer kind nor a key:
    // the runtime dispatches them from the one place focus changes, so drift
    // here is invisible on every other path and shows up only as an `on:blur`
    // that never runs.
    assert_eq!(name_binding::EVENT_FOCUS, view::EVENT_FOCUS);
    assert_eq!(name_binding::EVENT_BLUR, view::EVENT_BLUR);
    // The context request, which is emitted from one button of one pointer kind:
    // drift here is invisible everywhere else and shows up only as an
    // `on:contextmenu` that never runs under a right click.
    assert_eq!(name_binding::EVENT_CONTEXT_MENU, view::EVENT_CONTEXT_MENU);
}

#[test]
fn the_whole_reserved_event_set_is_the_same_set_on_both_sides() {
    // Naming the constants one at a time is what let `keydown` and `keyup` be
    // added to two crates and pinned in neither. The two lists are restated by
    // hand — `obscur-ui-ir` is a frontend crate and cannot depend on the view
    // tier — so pin the whole set, in order, from the one place that sees both.
    // A name added to one side and not the other fails here and nowhere else.
    assert_eq!(
        name_binding::RESERVED_EVENTS,
        view::RESERVED_EVENTS,
        "the two reserved-event tables have drifted"
    );
}

#[test]
fn every_reserved_event_has_an_authored_spelling_that_reaches_it() {
    // The other half of the same guard: a symbol both crates agree on is still
    // useless if no `on:` name lowers to it. Every reserved event must be
    // reachable from markup, and `is_reserved_event` must agree — that pair is
    // what lets a front end call `on:pointerdwon` a typo.
    for (spelling, symbol) in [
        ("click", view::EVENT_CLICK),
        ("input", view::EVENT_INPUT),
        ("change", view::EVENT_CHANGE),
        ("longpress", view::EVENT_LONG_PRESS),
        ("keydown", view::EVENT_KEY_DOWN),
        ("keyup", view::EVENT_KEY_UP),
        ("pointerdown", view::EVENT_POINTER_DOWN),
        ("pointerup", view::EVENT_POINTER_UP),
        ("pointermove", view::EVENT_POINTER_MOVE),
        ("pointercancel", view::EVENT_POINTER_CANCEL),
        ("pointerenter", view::EVENT_POINTER_ENTER),
        ("pointerleave", view::EVENT_POINTER_LEAVE),
        ("wheel", view::EVENT_WHEEL),
        ("focus", view::EVENT_FOCUS),
        ("blur", view::EVENT_BLUR),
        ("contextmenu", view::EVENT_CONTEXT_MENU),
    ] {
        assert_eq!(name_binding::event_symbol(spelling), symbol, "{spelling}");
        assert!(name_binding::is_reserved_event(spelling), "{spelling}");
    }
    // The DOM's bubbling crossings are deliberately NOT reserved: a command
    // routes either way here, so `pointerover` would be a second name for
    // `pointerenter`. They hash like any other author name, and nothing
    // dispatches them.
    for absent in ["pointerover", "pointerout"] {
        assert!(!name_binding::is_reserved_event(absent), "{absent}");
    }
    // The DOM's second focus pair is absent for a neighbouring reason: this
    // toolkit resolves an `on:focus` binding through the ancestor chain, so
    // `focus` here already *is* `focusin` and a second spelling would be a
    // second name for one event.
    for absent in ["focusin", "focusout"] {
        assert!(!name_binding::is_reserved_event(absent), "{absent}");
    }
    // `scroll` is not an alias for `wheel`, and not for want of one: the wheel
    // event fires *before* the scroll and can refuse it, so a name meaning "it
    // scrolled" would name a different event — one nothing emits.
    assert!(!name_binding::is_reserved_event("scroll"));
    // `menu` is not an alias for `contextmenu` either: this toolkit opens no
    // menu of its own, so a name meaning "a menu opened" would name an event
    // nothing emits — the exact defect this campaign exists to stop creating.
    assert!(!name_binding::is_reserved_event("menu"));
    // And the set is exactly that large — a reserved symbol with no spelling
    // would slip through the loop above unnoticed.
    //
    // 2026-08-03. This asserted 12, before `wheel` was reserved, then 13, before
    // `focus` / `blur` were, and then 15, before `contextmenu` was. The rule is
    // unchanged — the count must equal the number of spellings pinned above —
    // and the loop is what pins the pairing; this line only stops a symbol being
    // added with no spelling at all.
    assert_eq!(view::RESERVED_EVENTS.len(), 16);
}

#[test]
fn primitive_component_ids_match_their_source() {
    assert_eq!(name_binding::CONTAINER, Container::COMPONENT_ID);
    assert_eq!(name_binding::IMAGE, Image::COMPONENT_ID);
    assert_eq!(name_binding::SCROLL, Scroll::COMPONENT_ID);
    assert_eq!(name_binding::CANVAS, Canvas::COMPONENT_ID);
    assert_eq!(name_binding::PORTAL, Portal::COMPONENT_ID);
    assert_eq!(name_binding::LAYER, Layer::COMPONENT_ID);
    assert_eq!(name_binding::FOREIGN_SURFACE, ForeignSurface::COMPONENT_ID);
}

#[test]
fn widget_component_ids_match_their_source() {
    assert_eq!(name_binding::BUTTON, Button::COMPONENT_ID);
    assert_eq!(name_binding::INPUT, Input::COMPONENT_ID);
    assert_eq!(name_binding::CHECKBOX, Checkbox::COMPONENT_ID);
    assert_eq!(name_binding::SELECT, Select::COMPONENT_ID);
    assert_eq!(name_binding::MENU, Menu::COMPONENT_ID);
    assert_eq!(name_binding::DIALOG, Dialog::COMPONENT_ID);
    assert_eq!(name_binding::TABS, Tabs::COMPONENT_ID);
    assert_eq!(name_binding::LIST, List::COMPONENT_ID);
    assert_eq!(name_binding::TREE, Tree::COMPONENT_ID);
    assert_eq!(name_binding::TABLE, Table::COMPONENT_ID);
    assert_eq!(name_binding::DOCKING, Docking::COMPONENT_ID);
    assert_eq!(name_binding::RADIO, Radio::COMPONENT_ID);
}

#[test]
fn resolved_tags_bind_to_the_real_component_ids() {
    // The registry's resolver, not just its constants, lands on the real ids.
    assert_eq!(
        name_binding::component_id_for("Button", TagPosition::Component),
        Button::COMPONENT_ID
    );
    assert_eq!(
        name_binding::component_id_for("img", TagPosition::Element),
        Image::COMPONENT_ID
    );
    // The lowercase base-element spelling is the same widget as the component one:
    // HTML has a `<button>`, and so does this. It used to name no built-in, so it
    // fell through to `Container`.
    assert_eq!(
        name_binding::component_id_for("button", TagPosition::Element),
        Button::COMPONENT_ID
    );
    assert_eq!(
        name_binding::component_id_for("div", TagPosition::Element),
        Container::COMPONENT_ID
    );
    assert_eq!(name_binding::attr_symbol("src"), view::SRC);
    assert_eq!(name_binding::event_symbol("click"), view::EVENT_CLICK);
}

#[test]
fn every_control_the_ua_sheet_styles_resolves_to_the_shipped_widget() {
    // Cross-crate, so these are the real widget constants rather than the
    // frontend's mirror of them. Each of these controls used to answer to a
    // PascalCase tag only, so `<checkbox>`, `<input>`, `<select>` and `<table>`
    // fell through to `Container` — a styled box with no behaviour standing beside
    // the real control the Rust API builds, under one `checkbox { … }` rule that
    // reached both and meant two different things.
    for (element, id) in [
        ("button", Button::COMPONENT_ID),
        ("input", Input::COMPONENT_ID),
        ("checkbox", Checkbox::COMPONENT_ID),
        ("radio", Radio::COMPONENT_ID),
        ("select", Select::COMPONENT_ID),
        ("table", Table::COMPONENT_ID),
    ] {
        assert_eq!(
            name_binding::component_id_for(element, TagPosition::Element),
            id,
            "{element}"
        );
        assert_ne!(
            name_binding::component_id_for(element, TagPosition::Element),
            Container::COMPONENT_ID,
            "{element}"
        );
    }
    // HTML's two text-entry tags are one widget here — no `Textarea` type exists
    // to be a second one — and the multi-line/single-line distinction rides on the
    // local name instead, which is what `EditableField::is_multiline` reads.
    assert_eq!(
        name_binding::component_id_for("textarea", TagPosition::Element),
        Input::COMPONENT_ID
    );
    assert_eq!(
        name_binding::local_name_for("textarea", TagPosition::Element),
        Some("textarea")
    );
}

#[test]
fn an_authored_element_and_the_rust_widget_share_one_css_type() {
    // The local name `name_binding::local_name_for` gives an authored `<button>`
    // must be the SAME string `obscur_widgets::Button::view` sets under the
    // reserved LOCAL_NAME symbol — that is the whole reason `button { … }` can be written
    // once and style both. Drift here splits the UA stylesheet in two: the rule
    // would reach the Rust widget and not the markup, or the reverse, and nothing
    // else in either crate would notice.
    let node = Button::new("OK", obscur_kit::identity::InternedSymbol::named("save"))
        .view()
        .into_fragment()
        .nodes()
        .to_vec()
        .remove(0);
    let view::ViewNode::Element { props, .. } = &node else {
        panic!("a button's view is an element");
    };
    let widget_type = view::StyleAttributes::extract(props)
        .local_name()
        .map(str::to_owned);
    assert_eq!(widget_type.as_deref(), Some("button"));
    assert_eq!(
        name_binding::local_name_for("button", TagPosition::Element),
        widget_type.as_deref()
    );
    // And the symbol the frontend emits it under is the one the matcher reads.
    assert_eq!(name_binding::LOCAL_NAME, view::LOCAL_NAME);
}

#[doc = include_str!("okui_name_binding_drift.doc/widget_local_name.md")]
fn widget_local_name(rendered: view::View) -> Option<String> {
    let node = rendered.into_fragment().nodes().to_vec().remove(0);
    let view::ViewNode::Element { props, .. } = &node else {
        panic!("a widget's view root is an element");
    };
    view::StyleAttributes::extract(props)
        .local_name()
        .map(str::to_owned)
}

#[test]
fn every_control_and_its_tag_agree_on_one_css_type() {
    // The same claim as above, for the rest of the vocabulary the UA sheet is
    // written in. The tag and the widget must produce the SAME string, or the one
    // `input, textarea { … }` / `checkbox { … }` / `select { … }` rule styles one
    // of the two trees and silently misses the other.
    let save = obscur_kit::identity::InternedSymbol::named("save");
    for (tag, rendered) in [
        ("button", Button::new("OK", save).view()),
        ("input", Input::new(save).view()),
        ("textarea", Input::new(save).multiline(true).view()),
        ("checkbox", Checkbox::new("Agree", save).view()),
        ("radio", Radio::new("One", save).view()),
        ("select", Select::new(save).view()),
        ("table", Table::new().view()),
    ] {
        let widget_type = widget_local_name(rendered);
        assert_eq!(widget_type.as_deref(), Some(tag), "{tag}");
        assert_eq!(
            name_binding::local_name_for(tag, TagPosition::Element),
            widget_type.as_deref(),
            "{tag}"
        );
    }
}

// ---- widget contract drift (§2.13) ---------------------------------------
//
// `widget_contracts()` mirrors each widget's §2.6 builder surface by hand,
// because `obscur-okui-compiler` is a frontend crate and cannot depend on the
// components tier. A hand-mirrored list rots silently, so pin it from here —
// the one place that sees both sides.

#[test]
fn every_contracted_widget_tag_resolves_to_a_real_widget() {
    // A contract keyed by a tag the resolver does not know would never be
    // consulted: `component_id_for` would fall through to that tag's own
    // user-component id, which no shipped widget is registered under.
    let real: Vec<obscur_kit::identity::ComponentId> = vec![
        Button::COMPONENT_ID,
        Input::COMPONENT_ID,
        Checkbox::COMPONENT_ID,
        Select::COMPONENT_ID,
        Menu::COMPONENT_ID,
        Dialog::COMPONENT_ID,
        Tabs::COMPONENT_ID,
        List::COMPONENT_ID,
        Tree::COMPONENT_ID,
        Table::COMPONENT_ID,
        Docking::COMPONENT_ID,
        Radio::COMPONENT_ID,
    ];
    for contract in obscur_kit::okui_compiler::validation::widget_contracts() {
        let id = name_binding::component_id_for(contract.component(), TagPosition::Component);
        assert!(
            real.contains(&id),
            "contract tag {:?} does not resolve to a shipped widget (got {id:?})",
            contract.component()
        );
    }
}

#[test]
fn every_shipped_widget_has_a_contract() {
    // The other direction: adding a widget without a contract silently leaves
    // its authored surface unchecked — the exact gap this set was seeded to close.
    let contracts = obscur_kit::okui_compiler::validation::widget_contracts();
    let tags: Vec<&str> = contracts
        .iter()
        .map(obscur_kit::okui_compiler::validation::ComponentContract::component)
        .collect();
    for expected in [
        "Button", "Input", "Checkbox", "Select", "Menu", "Dialog", "Tabs", "List", "Tree", "Table",
        "Docking", "Radio",
    ] {
        assert!(
            tags.contains(&expected),
            "shipped widget {expected} has no contract"
        );
    }
}

#[test]
fn contracted_events_use_the_reserved_event_symbols() {
    // A contract may only allow events that actually have a reserved symbol;
    // anything else hashes to a fallback the runtime never dispatches, so
    // allowing it would bless a binding that silently does nothing.
    for contract in obscur_kit::okui_compiler::validation::widget_contracts() {
        for event in contract.allowed_events() {
            let symbol = name_binding::event_symbol(event);
            assert!(
                symbol == view::EVENT_CLICK
                    || symbol == view::EVENT_INPUT
                    || symbol == view::EVENT_CHANGE,
                "{} allows event {event:?}, which has no reserved symbol",
                contract.component()
            );
        }
    }
}

// ---- #[derive(Contract)] prop symbols (§2.13) -----------------------------
//
// A contract's prop symbol is only useful if it equals the symbol the desired
// view tree actually carries on that property. The derive computes it at
// expansion time; this pins the result against the real view-side constants.

#[derive(Debug, obscur_kit::ui_macros::Contract)]
struct ProbeProps {
    #[doc = include_str!("okui_name_binding_drift.doc/disabled.md")]
    disabled: bool,
    #[doc = include_str!("okui_name_binding_drift.doc/placeholder.md")]
    placeholder: String,
    #[doc = include_str!("okui_name_binding_drift.doc/label.md")]
    label: Option<String>,
}

#[test]
fn derived_prop_symbols_match_the_view_tree_symbols() {
    let contract = ProbeProps::contract();
    let symbols: Vec<_> = contract.props.iter().map(|p| p.name).collect();

    // The reserved name resolves to the shared constant, not to any hash.
    assert!(
        symbols.contains(&view::DISABLED),
        "derived `disabled` must be view::DISABLED"
    );
    // The non-reserved names resolve through the same registry the view uses.
    assert!(symbols.contains(&name_binding::attr_symbol("placeholder")));
    assert!(symbols.contains(&name_binding::attr_symbol("label")));
}

#[test]
fn a_derived_contract_can_be_checked_against_a_real_view_node() {
    // The end-to-end point: build a node the way the view layer does, and the
    // derived contract's symbols line up with the props it carries. Before the
    // symbols were unified this comparison could not succeed for `disabled`.
    use obscur_kit::view::view::{ViewNode, ViewProperty, ViewValue};

    let node = ViewNode::element(obscur_kit::identity::ComponentId::from_raw(1))
        .with_prop(ViewProperty::new(view::DISABLED, ViewValue::Bool(true)))
        .with_prop(ViewProperty::new(
            name_binding::attr_symbol("placeholder"),
            ViewValue::Text("type here".into()),
        ));
    let ViewNode::Element { props, .. } = &node else {
        panic!("an element");
    };

    let contract = ProbeProps::contract();
    for prop in &contract.props {
        // Every REQUIRED prop of the contract is findable on the node by symbol.
        if prop.required {
            assert!(
                props.iter().any(|p| p.name() == prop.name),
                "required prop {:?} not found on the node by symbol",
                prop.name
            );
        }
    }
}

// ---- #[derive(Contract)] generates the prop decoder (§3.1) ----------------
//
// `ComponentFactory` documented prop decoding as the missing keystone blocking an
// erased render ("depends on a prop-decoding story that does not exist yet").
// These prove the derive now emits it, against REAL view properties built the way
// the view layer builds them — the only check that would have caught the symbol
// drift the tests above pin.

#[test]
fn a_derived_props_struct_decodes_real_view_properties() {
    use obscur_kit::view::component::FromProps;
    use obscur_kit::view::view::{ViewProperty, ViewValue};

    let props = vec![
        ViewProperty::new(view::DISABLED, ViewValue::Bool(true)),
        ViewProperty::new(
            name_binding::attr_symbol("placeholder"),
            ViewValue::Text("type here".into()),
        ),
        ViewProperty::new(
            name_binding::attr_symbol("label"),
            ViewValue::Text("OK".into()),
        ),
    ];

    let decoded = ProbeProps::from_props(&props).expect("decodes");
    assert!(decoded.disabled);
    assert_eq!(decoded.placeholder, "type here");
    assert_eq!(decoded.label.as_deref(), Some("OK"));
}

#[test]
fn an_absent_optional_prop_is_none_and_an_absent_required_one_errors() {
    use obscur_kit::view::component::{FromProps, PropsError};
    use obscur_kit::view::view::{ViewProperty, ViewValue};

    // `label` is `Option<String>`, so omitting it is fine.
    let decoded = ProbeProps::from_props(&[
        ViewProperty::new(view::DISABLED, ViewValue::Bool(false)),
        ViewProperty::new(
            name_binding::attr_symbol("placeholder"),
            ViewValue::Text("x".into()),
        ),
    ])
    .expect("decodes without the optional prop");
    assert_eq!(decoded.label, None);

    // `disabled` is required: omitting it names the REAL view symbol.
    let error = ProbeProps::from_props(&[ViewProperty::new(
        name_binding::attr_symbol("placeholder"),
        ViewValue::Text("x".into()),
    )])
    .expect_err("a missing required prop is an error");
    assert_eq!(
        error,
        PropsError::Missing {
            name: view::DISABLED
        }
    );
}

#[test]
fn a_wrongly_typed_prop_is_an_error_not_a_silent_default() {
    use obscur_kit::view::component::{FromProps, PropsError};
    use obscur_kit::view::view::{ViewProperty, ViewValue};

    // An author writing `disabled="true"` (text, not bool) must be told.
    let error = ProbeProps::from_props(&[
        ViewProperty::new(view::DISABLED, ViewValue::Text("true".into())),
        ViewProperty::new(
            name_binding::attr_symbol("placeholder"),
            ViewValue::Text("x".into()),
        ),
    ])
    .expect_err("a wrongly typed prop is an error");
    assert_eq!(
        error,
        PropsError::Type {
            name: view::DISABLED,
            expected: "bool"
        }
    );
}
