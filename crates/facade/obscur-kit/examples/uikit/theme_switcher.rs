#![doc = include_str!("theme_switcher.doc/_module.md")]

use obscur_kit::identity::InternedSymbol;
use obscur_kit::view::view::{EVENT_CLICK, ViewEventBinding, ViewNode};

use super::classed_node::{el, text};
use super::theme_catalog::THEME_NAMES;

#[doc = include_str!("theme_switcher.doc/KIT_TAB_CMD.md")]
pub const KIT_TAB_CMD: [InternedSymbol; 8] = [
    InternedSymbol::from_raw(0x4B10),
    InternedSymbol::from_raw(0x4B11),
    InternedSymbol::from_raw(0x4B12),
    InternedSymbol::from_raw(0x4B13),
    InternedSymbol::from_raw(0x4B14),
    InternedSymbol::from_raw(0x4B15),
    InternedSymbol::from_raw(0x4B16),
    InternedSymbol::from_raw(0x4B17),
];

#[doc = include_str!("theme_switcher.doc/switcher.md")]
pub(super) fn switcher(active: usize) -> ViewNode {
    let brand = el(
        "kit-brand",
        vec![el("kit-logo", vec![]), text("kit-brand-name", "KIT")],
    );
    let tabs: Vec<ViewNode> = THEME_NAMES
        .iter()
        .enumerate()
        .map(|(i, label)| tab(label, i == active, KIT_TAB_CMD[i]))
        .collect();
    el("kit-switcher", vec![brand, el("kit-tabs", tabs)])
}

#[doc = include_str!("theme_switcher.doc/tab.md")]
fn tab(label: &str, active: bool, cmd: InternedSymbol) -> ViewNode {
    let mut kids = vec![text("kit-tab-label", label)];
    if active {
        kids.push(el("kit-underline", vec![]));
    }
    el(if active { "kit-tab active" } else { "kit-tab" }, kids)
        .with_event(ViewEventBinding::new(EVENT_CLICK, cmd))
}
