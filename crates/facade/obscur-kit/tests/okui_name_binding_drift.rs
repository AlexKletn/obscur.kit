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
