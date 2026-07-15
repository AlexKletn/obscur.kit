//! Drift guard: the frontend `name_binding` registry mirrors the raw handle
//! values of their canonical sources.
//!
//! `obscur-ui-ir::name_binding` must not depend on the component tier, so it
//! hard-codes the reserved `obscur-view` symbols and the widget/primitive
//! `COMPONENT_ID`s. This facade test — which *can* see every crate — pins each
//! mirrored constant to its real source, so a change to any canonical id fails
//! here instead of silently desynchronising the two `.okui` lowering paths.
#![forbid(unsafe_code)]

use obscur_kit::ui_ir::name_binding;
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
    assert_eq!(name_binding::SRC, view::SRC);
    assert_eq!(name_binding::DISABLED, view::DISABLED);
    assert_eq!(name_binding::CHECKED, view::CHECKED);
}

#[test]
fn reserved_event_symbols_match_obscur_view() {
    assert_eq!(name_binding::EVENT_CLICK, view::EVENT_CLICK);
    assert_eq!(name_binding::EVENT_INPUT, view::EVENT_INPUT);
    assert_eq!(name_binding::EVENT_CHANGE, view::EVENT_CHANGE);
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
    assert_eq!(name_binding::component_id_for("Button"), Button::COMPONENT_ID);
    assert_eq!(name_binding::component_id_for("img"), Image::COMPONENT_ID);
    assert_eq!(name_binding::component_id_for("div"), Container::COMPONENT_ID);
    assert_eq!(name_binding::attr_symbol("src"), view::SRC);
    assert_eq!(name_binding::event_symbol("click"), view::EVENT_CLICK);
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
    // consulted: `component_id_for` would fall through to CONTAINER.
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
        let id = name_binding::component_id_for(contract.component());
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
        "Button", "Input", "Checkbox", "Select", "Menu", "Dialog", "Tabs", "List", "Tree",
        "Table", "Docking", "Radio",
    ] {
        assert!(tags.contains(&expected), "shipped widget {expected} has no contract");
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
