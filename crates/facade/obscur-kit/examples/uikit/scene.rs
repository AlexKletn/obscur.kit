//! The **imported UI-Kit** scene: a top *theme-switcher* bar over a *Material 3*
//! component gallery, authored as ONE theme-agnostic view (semantic classes) plus
//! a swappable per-theme stylesheet. Rendered headless to a PNG by the sibling
//! `uikit_png.rs` example.
//!
//! # Architecture — one view, many stylesheets
//!
//! [`view`] builds the switcher + gallery using only *semantic* class names
//! (`.card`, `.btn`, `.btn-filled`, `.chip`, `.badge`, `.avatar`, `.slider`, …) —
//! it never names a Material color, radius, or shadow inline. All appearance lives
//! in the stylesheets returned by [`themes`]. Swapping to a different design system
//! later is therefore *pure CSS*: add one more `(name, css)` entry to [`themes`]
//! and author its rules against the same class vocabulary. **All eight** entries
//! are now authored in full — from the **Material** entry Phase 1 shipped through
//! the **Fluent / Acrylic** entry (index 7), which layers translucent surfaces
//! over a "mica" page wash and frosts them with `backdrop-filter: blur()`.
//!
//! # Fidelity notes (this is a faithfulness test of our CSS/layout/paint engine)
//!
//! The imported design uses a handful of things our markup/engine cannot take
//! verbatim; each is approximated and documented here:
//!
//! - **SVG icons** (FAB `+`, checkmarks, chevrons, kebab menu) → rendered as host
//!   font glyphs (`+`, `✓`, `▾`, `⋮`, `›`) — we never parse SVG.
//! - **Google Fonts** (Roboto / JetBrains Mono) → the host-provided CPU font
//!   (e.g. DejaVu). Text shapes and paints, just not in the exact face.
//! - **`letter-spacing`, `text-transform`, `line-height`** are not in our property
//!   set → dropped; eyebrow labels are authored already-uppercased.
//! - **`flex-wrap` / `flex: 1`** unsupported → rows use explicit widths + fixed
//!   flex tracks instead of wrapping.
//! - **Floating outlined-field labels** (label straddling the border) need overlap
//!   we don't do cleanly → labels are stacked above the value inside the box.
//! - **`repeating-linear-gradient`** cover → a plain `linear-gradient` approximation.
//! - **Two-layer `box-shadow`** IS supported (comma list) and used verbatim for the
//!   card elevation; GPU-class blur degrades to the CPU approximation per policy.
//! - The spinner is **static** (no animation on a still frame).
// One entity per file (xtask forbids logic in mod.rs); both examples could share
// this module, so some items look unused inside a single binary.
#![allow(dead_code, unreachable_pub)]

use obscur_kit::identity::InternedSymbol;
use obscur_kit::primitives::{Container, Text};
use obscur_kit::view::view::{
    CLASS, EVENT_CLICK, ID, View, ViewEventBinding, ViewNode, ViewProperty, ViewValue,
};

/// Index of the **Material** theme in [`themes`] — the active tab and the
/// stylesheet Phase 1 renders.
pub const MATERIAL_INDEX: usize = 1;

/// The eight reserved **click command** symbols, one per switcher tab: tab `i`'s
/// node binds `on-click → KIT_TAB_CMD[i]` (via [`EVENT_CLICK`]). These are purely
/// *bindings* — the static PNG examples never register a handler, so a click is a
/// no-op there; the windowed `uikit_window` example registers a handler per symbol
/// that live-swaps to `themes()[i]` and rebuilds `view(i)`. The values sit in an
/// app-owned command namespace (`0x4B10 + i`, "KIT tab i"), distinct from the
/// showcase scene's `0x50xx` commands, so nothing collides.
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

/// The eight theme tab labels, in bar order. Material (index [`MATERIAL_INDEX`]) is
/// the active tab and the only stylesheet authored in full for Phase 1.
pub const THEME_NAMES: [&str; 8] = [
    "Minimal",
    "Material",
    "Dark SaaS",
    "Flat",
    "Skeuomorphic",
    "Metro",
    "Lara",
    "Fluent",
];

/// The `(theme_name, stylesheet)` table. **All eight** slots are now authored in
/// full — Minimal (0, [`MINIMAL_CSS`]), Material ([`MATERIAL_INDEX`]), Dark SaaS (2,
/// [`DARK_SAAS_CSS`]), Flat (3, [`FLAT_CSS`]), Skeuomorphic (4, [`SKEUOMORPHIC_CSS`]),
/// Metro (5, [`METRO_CSS`]), Lara (6, [`LARA_CSS`]) and Fluent (7, [`FLUENT_CSS`],
/// the acrylic theme that exercises `backdrop-filter: blur()`) — each authored
/// against the same class vocabulary. Adding another theme = swapping one entry's
/// CSS. ([`NEUTRAL_CSS`] is kept only as a reference placeholder.)
#[must_use]
pub fn themes() -> &'static [(&'static str, &'static str)] {
    &[
        ("Minimal", MINIMAL_CSS),
        ("Material", MATERIAL_CSS),
        ("Dark SaaS", DARK_SAAS_CSS),
        ("Flat", FLAT_CSS),
        ("Skeuomorphic", SKEUOMORPHIC_CSS),
        ("Metro", METRO_CSS),
        ("Lara", LARA_CSS),
        ("Fluent", FLUENT_CSS),
    ]
}

/// Builds the full view: the switcher bar (with `active_theme` highlighted) above
/// the Material component gallery. Pure view/primitive construction — no colors,
/// no backend types — so any theme's stylesheet drives the identical tree.
#[must_use]
pub fn view(active_theme: usize) -> View {
    let app = Container::new()
        .with_children(vec![switcher(active_theme), gallery()])
        .into_view_node()
        .with_prop(ViewProperty::new(ID, ViewValue::Text("app".into())))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("app".into())));
    View::single(app)
}

// ---------------------------------------------------------------------------
// Switcher bar
// ---------------------------------------------------------------------------

/// The sticky top bar: a `KIT` brand mark then the row of eight theme tabs, the
/// `active` one brightened with a 2px underline.
fn switcher(active: usize) -> ViewNode {
    let brand = el("kit-brand", vec![el("kit-logo", vec![]), text("kit-brand-name", "KIT")]);
    let tabs: Vec<ViewNode> = THEME_NAMES
        .iter()
        .enumerate()
        .map(|(i, label)| tab(label, i == active, KIT_TAB_CMD[i]))
        .collect();
    el("kit-switcher", vec![brand, el("kit-tabs", tabs)])
}

/// One theme tab; the active tab gets the `active` class and an underline child.
/// The tab binds `on-click → cmd` ([`EVENT_CLICK`]) so a host runtime can resolve a
/// click on it to that command (the classes/CSS are unchanged; an unhandled command
/// is a no-op, so the static PNG examples keep rendering identically).
fn tab(label: &str, active: bool, cmd: InternedSymbol) -> ViewNode {
    let mut kids = vec![text("kit-tab-label", label)];
    if active {
        kids.push(el("kit-underline", vec![]));
    }
    el(if active { "kit-tab active" } else { "kit-tab" }, kids)
        .with_event(ViewEventBinding::new(EVENT_CLICK, cmd))
}

// ---------------------------------------------------------------------------
// Material gallery
// ---------------------------------------------------------------------------

fn gallery() -> ViewNode {
    let head = el(
        "m3-head",
        vec![
            text("m3-eyebrow", "STYLE 02"),
            text("m3-title", "Material 3"),
            text(
                "m3-sub",
                "Tonal color roles, state layers, generous rounding and elevation.",
            ),
        ],
    );

    let grid = el(
        "gallery-grid",
        vec![
            row("r1", vec![buttons_card(), selection_card()]),
            row("r2", vec![fields_card(), menu_card()]),
            row("r3", vec![cardlist_card(), navigation_card()]),
            row("r4", vec![badges_card(), sliders_card()]),
            table_card(),
            feedback_card(),
        ],
    );

    el("m3-page", vec![head, grid])
}

/// A gallery row: a flex track holding one or two half-width cards; the `variant`
/// class carries the row height so both cards stretch to it.
fn row(variant: &str, cards: Vec<ViewNode>) -> ViewNode {
    el(&format!("row {variant}"), cards)
}

/// A card shell: the mono eyebrow label then the card body children.
fn card(extra: &str, label: &str, mut body: Vec<ViewNode>) -> ViewNode {
    let mut kids = vec![text("card-label", label)];
    kids.append(&mut body);
    el(&format!("card {extra}"), kids)
}

fn buttons_card() -> ViewNode {
    let row1 = el(
        "btn-row",
        vec![
            btn("btn-filled", "Filled"),
            btn("btn-tonal", "Tonal"),
            btn("btn-outlined", "Outlined"),
            btn("btn-text", "Text"),
        ],
    );
    let row2 = el(
        "btn-row",
        vec![
            btn("btn-elevated", "Elevated"),
            btn("btn-disabled", "Disabled"),
            el("fab", vec![text("fab-glyph", "+")]),
        ],
    );
    card("col", "BUTTONS · STATES", vec![row1, row2])
}

/// A pill button: a flex box centering its label; the `variant` supplies color.
fn btn(variant: &str, label: &str) -> ViewNode {
    el(&format!("btn {variant}"), vec![text("btn-label", label)])
}

fn selection_card() -> ViewNode {
    let checks = el(
        "sel-col",
        vec![
            check_row("checkbox-on", vec![text("check-glyph", "✓")], "Checked"),
            check_row("checkbox-off", vec![], "Unchecked"),
            check_row("checkbox-disabled", vec![], "Disabled"),
        ],
    );
    let radios = el(
        "sel-col",
        vec![
            radio_row(true, "Standard"),
            radio_row(false, "Compact"),
            radio_row(false, "Spacious"),
        ],
    );
    let toggles = el(
        "sel-col wide-gap",
        vec![toggle_row(true, "On"), toggle_row(false, "Off")],
    );
    card("col", "SELECTION", vec![el("sel-cols", vec![checks, radios, toggles])])
}

fn check_row(boxclass: &str, inner: Vec<ViewNode>, label: &str) -> ViewNode {
    el(
        "sel-item",
        vec![el(boxclass, inner), text("sel-text", label)],
    )
}

fn radio_row(on: bool, label: &str) -> ViewNode {
    let ring = if on { "radio-on" } else { "radio-off" };
    let inner = if on { vec![el("radio-dot", vec![])] } else { vec![] };
    el("sel-item", vec![el(ring, inner), text("sel-text", label)])
}

fn toggle_row(on: bool, label: &str) -> ViewNode {
    let track = if on {
        el("toggle-on", vec![el("toggle-knob", vec![])])
    } else {
        el("toggle-off", vec![el("toggle-knob-off", vec![])])
    };
    el("sel-item", vec![track, text("sel-text", label)])
}

fn fields_card() -> ViewNode {
    let filled = el(
        "field field-filled",
        vec![
            text("field-label-primary", "Company name"),
            text("field-value", "Acme Industries"),
        ],
    );
    let outlined = el(
        "field field-outlined",
        vec![
            text("field-label", "Search"),
            text("field-placeholder", "Filter records…"),
        ],
    );
    let error = el(
        "field field-error",
        vec![
            text("field-label-error", "Email"),
            text("field-value", "not-an-email"),
            text("field-error-text", "Enter a valid address"),
        ],
    );
    card("col", "TEXT FIELDS", vec![el("field-col", vec![filled, outlined, error])])
}

fn menu_card() -> ViewNode {
    let select = el(
        "select-box",
        vec![text("select-value", "Medium"), text("select-chevron", "▾")],
    );
    let panel = el(
        "menu-panel",
        vec![
            el("menu-item", vec![text("menu-text", "Small")]),
            el(
                "menu-item selected",
                vec![text("menu-text", "Medium"), text("menu-check", "✓")],
            ),
            el("menu-item", vec![text("menu-text", "Large")]),
        ],
    );
    let chips = el(
        "chips-row",
        vec![
            el("chip chip-filled", vec![text("chip-check", "✓"), text("chip-text", "Design")]),
            el("chip chip-outlined", vec![text("chip-text", "Research")]),
            el("chip chip-outlined", vec![text("chip-text", "Ops")]),
        ],
    );
    card("col", "MENU · DROPDOWN", vec![select, panel, chips])
}

fn cardlist_card() -> ViewNode {
    let cover = el("mini-cover", vec![text("mini-cover-text", "cover image")]);
    let body = el(
        "mini-body",
        vec![
            text("mini-title", "Quarterly report"),
            text("mini-sub", "Updated 2 days ago · 14 pages"),
            el(
                "mini-actions",
                vec![btn("btn-filled btn-sm", "Open"), btn("btn-text btn-sm", "Share")],
            ),
        ],
    );
    let mini = el("mini-card", vec![cover, body]);
    let people = el(
        "people",
        vec![
            person("avatar-ak", "AK", "Anna Kovač", "anna@atlas.co"),
            person("avatar-mr", "MR", "Marko Rus", "marko@atlas.co"),
        ],
    );
    card("col", "CARD & LIST", vec![mini, people])
}

fn person(avatar_class: &str, initials: &str, name: &str, email: &str) -> ViewNode {
    el(
        "person",
        vec![
            el(&format!("avatar {avatar_class}"), vec![text("avatar-text", initials)]),
            el("person-meta", vec![text("person-name", name), text("person-email", email)]),
            text("person-menu", "⋮"),
        ],
    )
}

fn navigation_card() -> ViewNode {
    let crumbs = el(
        "breadcrumb",
        vec![
            text("crumb", "Workspace"),
            text("crumb-sep", "›"),
            text("crumb", "Projects"),
            text("crumb-sep", "›"),
            text("crumb-active", "Atlas"),
        ],
    );
    let tabs = el(
        "nav-tabs",
        vec![
            nav_tab("Overview", true),
            nav_tab("Activity", false),
            nav_tab("Settings", false),
        ],
    );
    let list = el(
        "nav-list",
        vec![
            nav_item(true, "dot-on", "Dashboard"),
            nav_item(false, "dot-off", "Members"),
            nav_item(false, "dot-off", "Billing"),
        ],
    );
    card("col", "NAVIGATION", vec![crumbs, tabs, list])
}

fn nav_tab(label: &str, active: bool) -> ViewNode {
    let mut kids = vec![text(if active { "nav-tab-label active" } else { "nav-tab-label" }, label)];
    if active {
        kids.push(el("nav-tab-underline", vec![]));
    }
    el("nav-tab", kids)
}

fn nav_item(active: bool, dot: &str, label: &str) -> ViewNode {
    el(
        if active { "nav-item active" } else { "nav-item" },
        vec![el(dot, vec![]), text("nav-item-text", label)],
    )
}

fn badges_card() -> ViewNode {
    let badges = el(
        "badge-row",
        vec![
            badge("badge-success", "Active"),
            badge("badge-warn", "Pending"),
            badge("badge-error", "Error"),
            el("badge-count", vec![text("badge-count-text", "12")]),
        ],
    );
    let stack = el(
        "avatar-stack",
        vec![
            stack_avatar("avatar-ak", "AK", false),
            stack_avatar("avatar-mr", "MR", true),
            stack_avatar("avatar-jp", "JP", true),
            stack_avatar("avatar-more", "+5", true),
        ],
    );
    card("col", "BADGES · CHIPS · AVATARS", vec![badges, stack])
}

fn badge(variant: &str, label: &str) -> ViewNode {
    el(&format!("badge {variant}"), vec![text("badge-text", label)])
}

fn stack_avatar(class: &str, initials: &str, overlap: bool) -> ViewNode {
    let cls = if overlap {
        format!("avatar stack {class} overlap")
    } else {
        format!("avatar stack {class}")
    };
    el(&cls, vec![text("avatar-text", initials)])
}

fn sliders_card() -> ViewNode {
    let head = el(
        "slider-head",
        vec![text("slider-name", "Volume"), text("slider-val", "55%")],
    );
    let slider = el(
        "slider",
        vec![
            el("slider-track", vec![]),
            el("slider-fill", vec![]),
            el("slider-knob", vec![]),
        ],
    );
    let progress_label = text("progress-label", "Linear · 72%");
    let progress = el("progress", vec![el("progress-fill", vec![])]);
    let spinner = el(
        "spinner-row",
        vec![el("spinner", vec![]), text("spinner-text", "Loading…")],
    );
    card("col", "SLIDERS · PROGRESS", vec![head, slider, progress_label, progress, spinner])
}

fn table_card() -> ViewNode {
    let header = el(
        "thead",
        vec![
            text("th", "Name"),
            text("th", "Role"),
            text("th", "Status"),
            text("th th-right", "MRR"),
        ],
    );
    let rows = vec![
        trow("Atlas Inc.", "Owner", "badge-success", "Active", "$4,200"),
        trow("Beacon LLC", "Editor", "badge-warn", "Pending", "$1,150"),
        trow("Cobalt Co.", "Viewer", "badge-neutral", "Draft", "$0"),
    ];
    let mut table_kids = vec![header];
    table_kids.extend(rows);
    let table = el("table", table_kids);
    card("full table-c", "DATA TABLE", vec![table])
}

fn trow(name: &str, role: &str, badge_variant: &str, status: &str, mrr: &str) -> ViewNode {
    el(
        "trow",
        vec![
            text("td td-name", name),
            text("td td-muted", role),
            el("td", vec![badge(badge_variant, status)]),
            text("td td-mono td-right", mrr),
        ],
    )
}

fn feedback_card() -> ViewNode {
    let row = el(
        "feedback-row",
        vec![
            btn("btn-filled btn-dialog", "Open dialog"),
            btn("btn-outlined btn-wide", "Show snackbar"),
            el("chip chip-tonal chip-wide", vec![text("chip-text", "Hover for tooltip")]),
        ],
    );
    card("full feedback-c", "FEEDBACK · OVERLAYS", vec![row])
}

// ---------------------------------------------------------------------------
// View-builder helpers (mirrors the showcase idiom)
// ---------------------------------------------------------------------------

/// Wraps `children` in a classed [`Container`] element node.
fn el(class: &str, children: Vec<ViewNode>) -> ViewNode {
    Container::new()
        .with_children(children)
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}

/// A classed [`Text`] node (the class carries color / size / weight).
fn text(class: &str, content: &str) -> ViewNode {
    Text::new(content.to_owned())
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}

// ---------------------------------------------------------------------------
// Stylesheets
// ---------------------------------------------------------------------------

/// The complete **Material 3** stylesheet — the only theme authored in full for
/// Phase 1. All appearance for the semantic classes lives here.
const MATERIAL_CSS: &str = r#"
* { box-sizing: border-box; }
#app { display: flex; flex-direction: column; background-color: #FEF7FF; }

/* ---- Switcher bar ---- */
.kit-switcher { display: flex; flex-direction: row; align-items: center; height: 60px;
    padding-left: 18px; padding-right: 18px; background-color: #16181f;
    border-bottom: 1px solid #262a36; }
.kit-brand { display: flex; flex-direction: row; align-items: center; width: 60px; height: 60px; }
.kit-logo { width: 11px; height: 11px; border-radius: 3px; background-color: #e8eaf0; }
.kit-brand-name { width: 40px; height: 16px; margin-left: 8px; color: #e8eaf0;
    font-size: 13px; font-weight: 700; }
.kit-tabs { display: flex; flex-direction: row; }
.kit-tab { display: flex; flex-direction: column; align-items: center; justify-content: center;
    width: 108px; height: 60px; }
.kit-tab-label { height: 16px; color: #9aa0b0; font-size: 12px; font-weight: 500; text-align: center; }
.kit-tab.active .kit-tab-label { color: #e8eaf0; }
.kit-underline { width: 60px; height: 2px; border-radius: 2px; background-color: #e8eaf0; margin-top: 6px; }

/* ---- Gallery frame ---- */
.m3-page { display: flex; flex-direction: column; padding: 34px 40px 60px; }
.m3-head { display: flex; flex-direction: column; height: 108px; margin-bottom: 26px; }
.m3-eyebrow { height: 16px; color: #7D7787; font-size: 11px; font-weight: 500; }
.m3-title { height: 48px; color: #1D1B20; font-size: 38px; font-weight: 400; }
.m3-sub { height: 22px; color: #49454F; font-size: 15px; }

.gallery-grid { display: flex; flex-direction: column; }
.row { display: flex; flex-direction: row; align-items: stretch; margin-bottom: 24px; }
.row.r1 { height: 232px; }
.row.r2 { height: 336px; }
.row.r3 { height: 420px; }
.row.r4 { height: 252px; }

.card { background-color: #ffffff; border-radius: 16px; padding: 22px;
    box-shadow: 0 1px 3px rgba(0,0,0,0.10), 0 1px 2px rgba(0,0,0,0.06); }
.card.col { width: 588px; margin-right: 24px; }
.card.full { width: 1200px; margin-bottom: 24px; }
.card.full.table-c { height: 288px; }
.card.full.feedback-c { height: 132px; }
.card-label { height: 16px; margin-bottom: 16px; color: #7D7787; font-size: 11px; font-weight: 500; }

/* ---- Buttons card ---- */
.btn-row { display: flex; flex-direction: row; align-items: center; height: 56px; }
.btn { display: flex; flex-direction: row; align-items: center; justify-content: center;
    height: 40px; border-radius: 100px; padding-left: 22px; padding-right: 22px; margin-right: 10px; }
.btn-label { height: 18px; font-size: 14px; font-weight: 500; text-align: center; }
.btn-filled { width: 110px; background-color: #6750A4; }
.btn-filled .btn-label { color: #ffffff; }
.btn-tonal { width: 104px; background-color: #E8DEF8; }
.btn-tonal .btn-label { color: #1D192B; }
.btn-outlined { width: 128px; background-color: #FEF7FF; border: 1px solid #79747E; }
.btn-outlined .btn-label { color: #6750A4; }
.btn-text { width: 92px; padding-left: 16px; padding-right: 16px; background-color: #FEF7FF; }
.btn-text .btn-label { color: #6750A4; }
.btn-dialog { width: 152px; }
.btn-elevated { width: 132px; background-color: #F7F2FA;
    box-shadow: 0 1px 3px rgba(0,0,0,0.18); }
.btn-elevated .btn-label { color: #6750A4; }
.btn-disabled { width: 128px; background-color: rgba(29,27,32,0.12); }
.btn-disabled .btn-label { color: rgba(29,27,32,0.38); }
.fab { display: flex; flex-direction: row; align-items: center; justify-content: center;
    width: 56px; height: 56px; border-radius: 18px; background-color: #EADDFF;
    box-shadow: 0 1px 3px rgba(0,0,0,0.20), 0 4px 8px rgba(0,0,0,0.12); }
.fab-glyph { height: 30px; color: #21005D; font-size: 26px; font-weight: 400; text-align: center; }
.btn-sm { height: 38px; }
.btn-wide { width: 168px; }

/* ---- Selection card ---- */
.sel-cols { display: flex; flex-direction: row; }
.sel-col { display: flex; flex-direction: column; width: 168px; }
.sel-item { display: flex; flex-direction: row; align-items: center; height: 34px; }
.sel-col .sel-item { margin-bottom: 14px; }
.sel-col.wide-gap .sel-item { margin-bottom: 18px; height: 32px; }
.sel-text { height: 20px; margin-left: 11px; color: #1D1B20; font-size: 14px; }
.checkbox-on { display: flex; align-items: center; justify-content: center;
    width: 18px; height: 18px; border-radius: 3px; background-color: #6750A4; }
.check-glyph { height: 14px; color: #ffffff; font-size: 12px; text-align: center; }
.checkbox-off { width: 18px; height: 18px; border-radius: 3px; border: 2px solid #49454F; }
.checkbox-disabled { width: 18px; height: 18px; border-radius: 3px; border: 2px solid rgba(29,27,32,0.20); }
.checkbox-disabled ~ .sel-text, .sel-item .checkbox-disabled + .sel-text { color: rgba(29,27,32,0.38); }
.radio-on { display: flex; align-items: center; justify-content: center;
    width: 20px; height: 20px; border-radius: 10px; border: 2px solid #6750A4; }
.radio-dot { width: 10px; height: 10px; border-radius: 5px; background-color: #6750A4; }
.radio-off { width: 20px; height: 20px; border-radius: 10px; border: 2px solid #79747E; }
.toggle-on { display: flex; flex-direction: row; align-items: center; justify-content: flex-end;
    width: 52px; height: 32px; border-radius: 16px; background-color: #6750A4; padding: 4px; }
.toggle-knob { width: 24px; height: 24px; border-radius: 12px; background-color: #ffffff; }
.toggle-off { display: flex; flex-direction: row; align-items: center; justify-content: flex-start;
    width: 52px; height: 32px; border-radius: 16px; background-color: #E6E0E9;
    border: 2px solid #79747E; padding: 6px; }
.toggle-knob-off { width: 16px; height: 16px; border-radius: 8px; background-color: #79747E; }

/* ---- Text fields card ---- */
.field-col { display: flex; flex-direction: column; }
.field { margin-bottom: 16px; }
.field-filled { background-color: #E7E0EC; border-radius: 4px 4px 0 0;
    border-bottom: 2px solid #6750A4; padding: 8px 14px 6px; }
.field-outlined { border: 1px solid #79747E; border-radius: 4px; padding: 10px 14px; }
.field-error { border: 2px solid #B3261E; border-radius: 4px; padding: 10px 14px; }
.field-label-primary { height: 14px; margin-bottom: 4px; color: #6750A4; font-size: 11px; }
.field-label { height: 14px; margin-bottom: 4px; color: #49454F; font-size: 11px; }
.field-label-error { height: 14px; margin-bottom: 4px; color: #B3261E; font-size: 11px; }
.field-value { height: 20px; color: #1D1B20; font-size: 15px; }
.field-placeholder { height: 20px; color: #79747E; font-size: 15px; }
.field-error-text { height: 14px; margin-top: 5px; color: #B3261E; font-size: 11px; }

/* ---- Menu card ---- */
.select-box { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    width: 260px; height: 44px; border: 1px solid #79747E; border-radius: 4px; padding: 0 14px; }
.select-value { height: 20px; color: #1D1B20; font-size: 15px; }
.select-chevron { width: 16px; height: 20px; color: #49454F; font-size: 14px; text-align: right; }
.menu-panel { display: flex; flex-direction: column; width: 260px; height: 136px; margin-top: 8px;
    background-color: #F7F2FA; border-radius: 8px; padding-top: 8px; padding-bottom: 8px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.18), 0 1px 3px rgba(0,0,0,0.12); }
.menu-item { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 40px; padding: 0 16px; }
.menu-item.selected { background-color: #EADDFF; }
.menu-text { height: 20px; color: #1D1B20; font-size: 15px; }
.menu-check { width: 16px; height: 20px; color: #6750A4; font-size: 14px; text-align: right; }
.chips-row { display: flex; flex-direction: row; align-items: center; height: 32px; margin-top: 16px; }
.chip { display: flex; flex-direction: row; align-items: center; justify-content: center; height: 32px;
    border-radius: 8px; padding-left: 14px; padding-right: 14px; margin-right: 8px; }
.chip-filled { width: 112px; background-color: #E8DEF8; }
.chip-filled .chip-text { color: #1D192B; }
.chip-check { height: 16px; margin-right: 6px; color: #1D192B; font-size: 12px; }
.chip-outlined { width: 104px; border: 1px solid #79747E; }
.chip-outlined .chip-text { color: #49454F; }
.chip-tonal { background-color: #E8DEF8; }
.chip-tonal .chip-text { color: #1D192B; }
.chip-wide { width: 172px; }
.chip-text { height: 18px; font-size: 14px; font-weight: 500; }

/* ---- Card & list ---- */
.mini-card { display: flex; flex-direction: column; height: 208px; background-color: #F7F2FA;
    border-radius: 12px; box-shadow: 0 1px 3px rgba(0,0,0,0.12); }
.mini-cover { display: flex; align-items: center; justify-content: center; height: 84px;
    border-radius: 12px 12px 0 0;
    background-image: linear-gradient(135deg, #E8DEF8, #EADDFF); }
.mini-cover-text { height: 16px; color: #6750A4; font-size: 11px; text-align: center; }
.mini-body { display: flex; flex-direction: column; height: 124px; padding: 16px; }
.mini-title { height: 22px; color: #1D1B20; font-size: 16px; font-weight: 500; }
.mini-sub { height: 18px; margin-top: 4px; color: #49454F; font-size: 13px; }
.mini-actions { display: flex; flex-direction: row; align-items: center; height: 38px; margin-top: 14px; }
.people { display: flex; flex-direction: column; height: 100px; margin-top: 14px; }
.person { display: flex; flex-direction: row; align-items: center; height: 50px; }
.person-meta { display: flex; flex-direction: column; justify-content: center; height: 50px;
    margin-left: 13px; width: 420px; }
.person-name { height: 18px; color: #1D1B20; font-size: 14px; }
.person-email { height: 16px; margin-top: 2px; color: #49454F; font-size: 12px; }
.person-menu { width: 24px; height: 20px; color: #79747E; font-size: 18px; text-align: right; }
.avatar { display: flex; align-items: center; justify-content: center;
    width: 36px; height: 36px; border-radius: 18px; }
.avatar-text { height: 16px; font-size: 13px; font-weight: 500; text-align: center; }
.avatar-ak { background-color: #6750A4; }
.avatar-ak .avatar-text { color: #ffffff; }
.avatar-mr { background-color: #7D5260; }
.avatar-mr .avatar-text { color: #ffffff; }
.avatar-jp { background-color: #386641; }
.avatar-jp .avatar-text { color: #ffffff; }
.avatar-more { background-color: #E8DEF8; }
.avatar-more .avatar-text { color: #6750A4; }

/* ---- Navigation ---- */
.breadcrumb { display: flex; flex-direction: row; align-items: center; height: 20px; margin-bottom: 18px; }
.crumb { height: 18px; margin-right: 8px; color: #49454F; font-size: 13px; }
.crumb-sep { height: 18px; margin-right: 8px; color: #CAC4D0; font-size: 13px; }
.crumb-active { height: 18px; color: #6750A4; font-size: 13px; font-weight: 500; }
.nav-tabs { display: flex; flex-direction: row; height: 42px;
    border-bottom: 1px solid #E7E0EC; margin-bottom: 14px; }
.nav-tab { display: flex; flex-direction: column; align-items: center; justify-content: flex-end;
    width: 96px; }
.nav-tab-label { height: 24px; color: #49454F; font-size: 14px; font-weight: 500; text-align: center; }
.nav-tab-label.active { color: #6750A4; }
.nav-tab-underline { width: 72px; height: 3px; border-radius: 3px 3px 0 0; background-color: #6750A4; }
.nav-list { display: flex; flex-direction: column; width: 260px; }
.nav-item { display: flex; flex-direction: row; align-items: center; height: 44px;
    border-radius: 100px; padding-left: 16px; padding-right: 16px; margin-bottom: 4px; }
.nav-item.active { background-color: #E8DEF8; }
.nav-item-text { height: 20px; margin-left: 12px; color: #49454F; font-size: 14px; }
.nav-item.active .nav-item-text { color: #1D192B; font-weight: 500; }
.dot-on { width: 8px; height: 8px; border-radius: 4px; background-color: #6750A4; }
.dot-off { width: 8px; height: 8px; border-radius: 4px; background-color: #CAC4D0; }

/* ---- Badges / chips / avatars ---- */
.badge-row { display: flex; flex-direction: row; align-items: center; height: 26px; margin-bottom: 22px; }
.badge { display: flex; align-items: center; justify-content: center; height: 26px;
    border-radius: 8px; margin-right: 8px; }
.badge-text { height: 16px; font-size: 12px; font-weight: 500; text-align: center; }
.badge-success { width: 76px; background-color: #D5F2DF; }
.badge-success .badge-text { color: #0F5132; }
.badge-warn { width: 84px; background-color: #FCE8C9; }
.badge-warn .badge-text { color: #7A4F01; }
.badge-error { width: 66px; background-color: #F9DEDC; }
.badge-error .badge-text { color: #B3261E; }
.badge-neutral { width: 64px; background-color: #E7E0EC; }
.badge-neutral .badge-text { color: #49454F; }
.badge-count { display: flex; align-items: center; justify-content: center;
    width: 24px; height: 20px; border-radius: 10px; background-color: #B3261E; }
.badge-count-text { height: 14px; color: #ffffff; font-size: 11px; font-weight: 700; text-align: center; }
.avatar-stack { display: flex; flex-direction: row; align-items: center; height: 40px; }
.avatar.stack { border: 2px solid #ffffff; }
.avatar.overlap { margin-left: -10px; }

/* ---- Sliders / progress ---- */
.slider-head { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 18px; margin-bottom: 10px; }
.slider-name { height: 16px; color: #49454F; font-size: 13px; }
.slider-val { height: 16px; color: #6750A4; font-size: 13px; text-align: right; }
.slider { position: relative; height: 24px; margin-bottom: 24px; }
.slider-track { position: absolute; left: 0; right: 0; top: 10px; height: 4px;
    border-radius: 2px; background-color: #E7E0EC; }
.slider-fill { position: absolute; left: 0; top: 10px; width: 55%; height: 4px;
    border-radius: 2px; background-color: #6750A4; }
.slider-knob { position: absolute; left: 53%; top: 2px; width: 20px; height: 20px;
    border-radius: 10px; background-color: #6750A4; }
.progress-label { height: 18px; margin-bottom: 8px; color: #49454F; font-size: 13px; }
.progress { height: 4px; border-radius: 2px; background-color: #E7E0EC; margin-bottom: 20px; }
.progress-fill { width: 72%; height: 4px; border-radius: 2px; background-color: #6750A4; }
.spinner-row { display: flex; flex-direction: row; align-items: center; height: 28px; }
.spinner { width: 28px; height: 28px; border-radius: 14px; margin-right: 14px;
    border: 3px solid #E7E0EC; border-top-color: #6750A4; }
.spinner-text { height: 18px; color: #49454F; font-size: 13px; }

/* ---- Data table ---- */
.table { border: 1px solid #E7E0EC; border-radius: 12px; }
.thead { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr;
    background-color: #F7F2FA; padding: 14px 18px; border-radius: 12px 12px 0 0; }
.th { height: 16px; color: #49454F; font-size: 12px; font-weight: 500; }
.th-right { text-align: right; }
.trow { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr; align-items: center;
    padding: 14px 18px; border-top: 1px solid #E7E0EC; }
.td { height: 20px; color: #1D1B20; font-size: 14px; }
.td-name { font-weight: 500; }
.td-muted { color: #49454F; }
.td-mono { color: #1D1B20; }
.td-right { text-align: right; }

/* ---- Feedback ---- */
.feedback-row { display: flex; flex-direction: row; align-items: center; height: 40px; }
"#;

/// A neutral placeholder stylesheet for the seven not-yet-authored themes. It
/// styles the same semantic classes plainly so a theme swap still produces a
/// coherent (if unstyled) frame. Filling in a real theme = replacing this string
/// in the corresponding [`themes`] entry.
const NEUTRAL_CSS: &str = r#"
* { box-sizing: border-box; }
#app { display: flex; flex-direction: column; background-color: #f4f4f5; }
.kit-switcher { display: flex; flex-direction: row; align-items: center; height: 60px;
    padding-left: 18px; padding-right: 18px; background-color: #e4e4e7;
    border-bottom: 1px solid #d4d4d8; }
.kit-brand { display: flex; flex-direction: row; align-items: center; width: 60px; height: 60px; }
.kit-logo { width: 11px; height: 11px; border-radius: 3px; background-color: #52525b; }
.kit-brand-name { width: 40px; height: 16px; margin-left: 8px; color: #27272a; font-size: 13px; font-weight: 700; }
.kit-tabs { display: flex; flex-direction: row; }
.kit-tab { display: flex; flex-direction: column; align-items: center; justify-content: center;
    width: 108px; height: 60px; }
.kit-tab-label { height: 16px; color: #71717a; font-size: 12px; font-weight: 500; text-align: center; }
.kit-tab.active .kit-tab-label { color: #18181b; }
.kit-underline { width: 60px; height: 2px; border-radius: 2px; background-color: #18181b; margin-top: 6px; }
.m3-page { display: flex; flex-direction: column; padding: 34px 40px 60px; }
.m3-head { display: flex; flex-direction: column; height: 108px; margin-bottom: 26px; }
.m3-eyebrow { height: 16px; color: #71717a; font-size: 11px; font-weight: 500; }
.m3-title { height: 48px; color: #18181b; font-size: 38px; font-weight: 700; }
.m3-sub { height: 22px; color: #52525b; font-size: 15px; }
.gallery-grid { display: flex; flex-direction: column; }
.row { display: flex; flex-direction: row; align-items: stretch; margin-bottom: 24px; }
.row.r1 { height: 210px; }
.row.r2 { height: 246px; }
.row.r3 { height: 320px; }
.row.r4 { height: 214px; }
.card { background-color: #ffffff; border-radius: 8px; padding: 22px; border: 1px solid #e4e4e7; }
.card.col { width: 588px; margin-right: 24px; }
.card.full { width: 1200px; margin-bottom: 24px; }
.card-label { height: 16px; margin-bottom: 16px; color: #71717a; font-size: 11px; font-weight: 600; }
.btn-row { display: flex; flex-direction: row; align-items: center; height: 56px; }
.btn { display: flex; align-items: center; justify-content: center; height: 36px; border-radius: 6px;
    padding-left: 16px; padding-right: 16px; margin-right: 10px; background-color: #e4e4e7; }
.btn-label { height: 18px; font-size: 14px; font-weight: 500; text-align: center; color: #27272a; }
"#;

/// The complete **Minimal / Neutral** stylesheet (theme index 0) — an airy, light
/// "Helvetica" system: warm off-white page (`#f6f6f3`), white cards with a single
/// hairline border (`1px #e7e7e1`) and *no* shadow, a small 8px radius, one calm
/// muted-blue accent (`#407cc5`, the spec's `oklch(0.58 0.13 255)` converted to
/// sRGB), and lots of whitespace. Authored against the same semantic classes as
/// [`MATERIAL_CSS`]; the layout numbers (card widths, row heights, control sizes)
/// are shared so the fixed view fits. Restraint is the whole aesthetic: flat solid
/// fills only, so this reproduces faithfully on the CPU backend.
///
/// # Approximated (our engine's known gaps)
/// - **`letter-spacing` / `text-transform`** on the mono eyebrow labels are dropped
///   (labels are already uppercased in the view); no engine support.
/// - The card-cover **`repeating-linear-gradient` hatch** is approximated by a flat
///   light fill (`background-color`), matching the CPU effect-split policy.
const MINIMAL_CSS: &str = r#"
* { box-sizing: border-box; }
#app { display: flex; flex-direction: column; background-color: #f6f6f3; }

/* ---- Switcher bar: light + airy (Minimal inverts Material's dark strip) ---- */
.kit-switcher { display: flex; flex-direction: row; align-items: center; height: 60px;
    padding-left: 18px; padding-right: 18px; background-color: #ffffff;
    border-bottom: 1px solid #e7e7e1; }
.kit-brand { display: flex; flex-direction: row; align-items: center; width: 60px; height: 60px; }
.kit-logo { width: 11px; height: 11px; border-radius: 4px; background-color: #407cc5; }
.kit-brand-name { width: 40px; height: 16px; margin-left: 8px; color: #1b1b18;
    font-size: 13px; font-weight: 700; }
.kit-tabs { display: flex; flex-direction: row; }
.kit-tab { display: flex; flex-direction: column; align-items: center; justify-content: center;
    width: 108px; height: 60px; }
.kit-tab-label { height: 16px; color: #9a9a92; font-size: 12px; font-weight: 500; text-align: center; }
.kit-tab.active .kit-tab-label { color: #1b1b18; }
.kit-underline { width: 60px; height: 2px; border-radius: 2px; background-color: #407cc5; margin-top: 6px; }

/* ---- Gallery frame ---- */
.m3-page { display: flex; flex-direction: column; padding: 34px 40px 60px; }
.m3-head { display: flex; flex-direction: column; height: 108px; margin-bottom: 26px; }
.m3-eyebrow { height: 16px; color: #a3a399; font-size: 11px; font-weight: 500; }
.m3-title { height: 48px; color: #1b1b18; font-size: 38px; font-weight: 600; }
.m3-sub { height: 22px; color: #76766d; font-size: 15px; }

.gallery-grid { display: flex; flex-direction: column; }
.row { display: flex; flex-direction: row; align-items: stretch; margin-bottom: 24px; }
.row.r1 { height: 232px; }
.row.r2 { height: 336px; }
.row.r3 { height: 420px; }
.row.r4 { height: 252px; }

/* Cards: white with ONE hairline border, small radius, no shadow. */
.card { background-color: #ffffff; border-radius: 14px; padding: 22px;
    border: 1px solid #e7e7e1; }
.card.col { width: 588px; margin-right: 24px; }
.card.full { width: 1200px; margin-bottom: 24px; }
.card.full.table-c { height: 288px; }
.card.full.feedback-c { height: 132px; }
.card-label { height: 16px; margin-bottom: 16px; color: #a3a399; font-size: 11px; font-weight: 500; }

/* ---- Buttons: 8px rounded rects, one accent, otherwise white+border ---- */
.btn-row { display: flex; flex-direction: row; align-items: center; height: 56px; }
.btn { display: flex; flex-direction: row; align-items: center; justify-content: center;
    height: 40px; border-radius: 8px; padding-left: 20px; padding-right: 20px; margin-right: 10px; }
.btn-label { height: 18px; font-size: 14px; font-weight: 500; text-align: center; }
.btn-filled { width: 110px; background-color: #407cc5; }
.btn-filled .btn-label { color: #ffffff; }
.btn-tonal { width: 104px; background-color: #f3f3ee; border: 1px solid #e7e7e1; }
.btn-tonal .btn-label { color: #26261f; }
.btn-outlined { width: 128px; background-color: #ffffff; border: 1px solid #d9d9d1; }
.btn-outlined .btn-label { color: #26261f; }
.btn-text { width: 92px; padding-left: 16px; padding-right: 16px; background-color: #ffffff; }
.btn-text .btn-label { color: #3568b0; }
.btn-dialog { width: 152px; }
.btn-elevated { width: 132px; background-color: #ffffff; border: 1px solid #e7e7e1;
    box-shadow: 0 1px 2px rgba(20,20,16,0.06); }
.btn-elevated .btn-label { color: #26261f; }
.btn-disabled { width: 128px; background-color: #f3f3ee; border: 1px solid #e7e7e1; }
.btn-disabled .btn-label { color: #bcbcb2; }
/* FAB → Minimal has no floating action button; render the icon-square affordance. */
.fab { display: flex; flex-direction: row; align-items: center; justify-content: center;
    width: 56px; height: 56px; border-radius: 8px; background-color: #ffffff; border: 1px solid #d9d9d1; }
.fab-glyph { height: 30px; color: #5a5a52; font-size: 26px; font-weight: 400; text-align: center; }
.btn-sm { height: 38px; }
.btn-wide { width: 168px; }

/* ---- Selection ---- */
.sel-cols { display: flex; flex-direction: row; }
.sel-col { display: flex; flex-direction: column; width: 168px; }
.sel-item { display: flex; flex-direction: row; align-items: center; height: 34px; }
.sel-col .sel-item { margin-bottom: 14px; }
.sel-col.wide-gap .sel-item { margin-bottom: 18px; height: 32px; }
.sel-text { height: 20px; margin-left: 11px; color: #1b1b18; font-size: 14px; }
.checkbox-on { display: flex; align-items: center; justify-content: center;
    width: 18px; height: 18px; border-radius: 6px; background-color: #407cc5; }
.check-glyph { height: 14px; color: #ffffff; font-size: 12px; text-align: center; }
.checkbox-off { width: 18px; height: 18px; border-radius: 6px; border: 1px solid #cccbc1; }
.checkbox-disabled { width: 18px; height: 18px; border-radius: 6px; border: 1px solid #e4e4de;
    background-color: #f3f3ee; }
.radio-on { display: flex; align-items: center; justify-content: center;
    width: 20px; height: 20px; border-radius: 10px; border: 1px solid #cccbc1; }
.radio-dot { width: 10px; height: 10px; border-radius: 5px; background-color: #407cc5; }
.radio-off { width: 20px; height: 20px; border-radius: 10px; border: 1px solid #cccbc1; }
.toggle-on { display: flex; flex-direction: row; align-items: center; justify-content: flex-end;
    width: 44px; height: 24px; border-radius: 12px; background-color: #407cc5; padding: 3px; }
.toggle-knob { width: 18px; height: 18px; border-radius: 9px; background-color: #ffffff;
    box-shadow: 0 1px 2px rgba(0,0,0,0.25); }
.toggle-off { display: flex; flex-direction: row; align-items: center; justify-content: flex-start;
    width: 44px; height: 24px; border-radius: 12px; background-color: #d6d6cd; padding: 3px; }
.toggle-knob-off { width: 18px; height: 18px; border-radius: 9px; background-color: #ffffff;
    box-shadow: 0 1px 2px rgba(0,0,0,0.25); }

/* ---- Text fields: soft off-white wells, hairline border, 8px radius ---- */
.field-col { display: flex; flex-direction: column; }
.field { margin-bottom: 16px; }
.field-filled { background-color: #fafaf7; border-radius: 8px; border: 1px solid #d9d9d1; padding: 9px 12px; }
.field-outlined { background-color: #fafaf7; border: 1px solid #d9d9d1; border-radius: 8px; padding: 9px 12px; }
.field-error { background-color: #fdf4f3; border: 1px solid #d97064; border-radius: 8px; padding: 9px 12px; }
.field-label-primary { height: 14px; margin-bottom: 4px; color: #76766d; font-size: 11px; }
.field-label { height: 14px; margin-bottom: 4px; color: #76766d; font-size: 11px; }
.field-label-error { height: 14px; margin-bottom: 4px; color: #c0392b; font-size: 11px; }
.field-value { height: 20px; color: #1b1b18; font-size: 15px; }
.field-placeholder { height: 20px; color: #a3a399; font-size: 15px; }
.field-error-text { height: 14px; margin-top: 5px; color: #c0392b; font-size: 11px; }

/* ---- Menu / dropdown ---- */
.select-box { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    width: 260px; height: 44px; background-color: #fafaf7; border: 1px solid #d9d9d1; border-radius: 8px; padding: 0 14px; }
.select-value { height: 20px; color: #1b1b18; font-size: 15px; }
.select-chevron { width: 16px; height: 20px; color: #a3a399; font-size: 14px; text-align: right; }
.menu-panel { display: flex; flex-direction: column; width: 260px; height: 136px; margin-top: 8px;
    background-color: #ffffff; border: 1px solid #e4e4de; border-radius: 8px; padding-top: 6px; padding-bottom: 6px;
    box-shadow: 0 8px 24px rgba(20,20,16,0.10); }
.menu-item { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 40px; padding: 0 14px; }
.menu-item.selected { background-color: #f3f3ee; border-radius: 6px; }
.menu-text { height: 20px; color: #1b1b18; font-size: 15px; }
.menu-check { width: 16px; height: 20px; color: #407cc5; font-size: 14px; text-align: right; }
.chips-row { display: flex; flex-direction: row; align-items: center; height: 32px; margin-top: 16px; }
.chip { display: flex; flex-direction: row; align-items: center; justify-content: center; height: 32px;
    border-radius: 8px; padding-left: 12px; padding-right: 12px; margin-right: 8px; }
.chip-filled { width: 112px; background-color: #f3f3ee; border: 1px solid #e4e4de; }
.chip-filled .chip-text { color: #26261f; }
.chip-check { height: 16px; margin-right: 6px; color: #76766d; font-size: 12px; }
.chip-outlined { width: 104px; background-color: #f3f3ee; border: 1px solid #e4e4de; }
.chip-outlined .chip-text { color: #26261f; }
.chip-tonal { background-color: #f3f3ee; border: 1px solid #e4e4de; }
.chip-tonal .chip-text { color: #26261f; }
.chip-wide { width: 172px; }
.chip-text { height: 18px; font-size: 14px; font-weight: 500; }

/* ---- Card & list ---- */
.mini-card { display: flex; flex-direction: column; height: 208px; background-color: #ffffff;
    border: 1px solid #ebebe4; border-radius: 12px; }
.mini-cover { display: flex; align-items: center; justify-content: center; height: 84px;
    border-radius: 12px 12px 0 0; background-color: #eeeee7; }
.mini-cover-text { height: 16px; color: #b6b6ac; font-size: 11px; text-align: center; }
.mini-body { display: flex; flex-direction: column; height: 124px; padding: 16px; }
.mini-title { height: 22px; color: #1b1b18; font-size: 16px; font-weight: 600; }
.mini-sub { height: 18px; margin-top: 4px; color: #76766d; font-size: 13px; }
.mini-actions { display: flex; flex-direction: row; align-items: center; height: 38px; margin-top: 14px; }
.people { display: flex; flex-direction: column; height: 100px; margin-top: 14px; }
.person { display: flex; flex-direction: row; align-items: center; height: 50px; }
.person-meta { display: flex; flex-direction: column; justify-content: center; height: 50px;
    margin-left: 13px; width: 420px; }
.person-name { height: 18px; color: #1b1b18; font-size: 14px; }
.person-email { height: 16px; margin-top: 2px; color: #76766d; font-size: 12px; }
.person-menu { width: 24px; height: 20px; color: #a3a399; font-size: 18px; text-align: right; }
.avatar { display: flex; align-items: center; justify-content: center;
    width: 36px; height: 36px; border-radius: 18px; }
.avatar-text { height: 16px; font-size: 13px; font-weight: 600; text-align: center; color: #ffffff; }
.avatar-ak { background-color: #d58679; }
.avatar-mr { background-color: #6fb07d; }
.avatar-jp { background-color: #7a9fdd; }
.avatar-more { background-color: #e9e9e2; }
.avatar-more .avatar-text { color: #76766d; }

/* ---- Navigation ---- */
.breadcrumb { display: flex; flex-direction: row; align-items: center; height: 20px; margin-bottom: 18px; }
.crumb { height: 18px; margin-right: 8px; color: #76766d; font-size: 13px; }
.crumb-sep { height: 18px; margin-right: 8px; color: #cccbc1; font-size: 13px; }
.crumb-active { height: 18px; color: #1b1b18; font-size: 13px; font-weight: 500; }
.nav-tabs { display: flex; flex-direction: row; height: 42px;
    border-bottom: 1px solid #ebebe4; margin-bottom: 14px; }
.nav-tab { display: flex; flex-direction: column; align-items: center; justify-content: flex-end;
    width: 96px; }
.nav-tab-label { height: 24px; color: #9a9a90; font-size: 14px; font-weight: 500; text-align: center; }
.nav-tab-label.active { color: #1b1b18; }
.nav-tab-underline { width: 72px; height: 2px; border-radius: 2px; background-color: #407cc5; }
.nav-list { display: flex; flex-direction: column; width: 260px; }
.nav-item { display: flex; flex-direction: row; align-items: center; height: 40px;
    border-radius: 7px; padding-left: 12px; padding-right: 12px; margin-bottom: 4px; }
.nav-item.active { background-color: #eaf0f9; }
.nav-item-text { height: 20px; margin-left: 10px; color: #5a5a52; font-size: 14px; }
.nav-item.active .nav-item-text { color: #2b5697; font-weight: 500; }
.dot-on { width: 6px; height: 6px; border-radius: 3px; background-color: #407cc5; }
.dot-off { width: 6px; height: 6px; border-radius: 3px; background-color: #cccbc1; }

/* ---- Badges / chips / avatars: muted pastel pills ---- */
.badge-row { display: flex; flex-direction: row; align-items: center; height: 26px; margin-bottom: 22px; }
.badge { display: flex; align-items: center; justify-content: center; height: 24px;
    border-radius: 20px; margin-right: 8px; }
.badge-text { height: 16px; font-size: 12px; font-weight: 500; text-align: center; }
.badge-success { width: 76px; background-color: #e6f4ea; }
.badge-success .badge-text { color: #0f6634; }
.badge-warn { width: 84px; background-color: #f3eeda; }
.badge-warn .badge-text { color: #7a4f01; }
.badge-error { width: 66px; background-color: #f7e9e5; }
.badge-error .badge-text { color: #a83634; }
.badge-neutral { width: 64px; background-color: #efefe8; }
.badge-neutral .badge-text { color: #76766d; }
.badge-count { display: flex; align-items: center; justify-content: center;
    width: 24px; height: 20px; border-radius: 6px; background-color: #407cc5; }
.badge-count-text { height: 14px; color: #ffffff; font-size: 11px; font-weight: 700; text-align: center; }
.avatar-stack { display: flex; flex-direction: row; align-items: center; height: 40px; }
.avatar.stack { border: 2px solid #ffffff; }
.avatar.overlap { margin-left: -9px; }

/* ---- Sliders / progress ---- */
.slider-head { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 18px; margin-bottom: 10px; }
.slider-name { height: 16px; color: #76766d; font-size: 13px; }
.slider-val { height: 16px; color: #1b1b18; font-size: 13px; text-align: right; }
.slider { position: relative; height: 24px; margin-bottom: 24px; }
.slider-track { position: absolute; left: 0; right: 0; top: 10px; height: 5px;
    border-radius: 3px; background-color: #e4e4de; }
.slider-fill { position: absolute; left: 0; top: 10px; width: 55%; height: 5px;
    border-radius: 3px; background-color: #407cc5; }
.slider-knob { position: absolute; left: 53%; top: 4px; width: 17px; height: 17px;
    border-radius: 9px; background-color: #ffffff; border: 1px solid #cccbc1;
    box-shadow: 0 1px 3px rgba(0,0,0,0.18); }
.progress-label { height: 18px; margin-bottom: 8px; color: #76766d; font-size: 13px; }
.progress { height: 8px; border-radius: 4px; background-color: #e4e4de; margin-bottom: 20px; }
.progress-fill { width: 72%; height: 8px; border-radius: 4px; background-color: #407cc5; }
.spinner-row { display: flex; flex-direction: row; align-items: center; height: 28px; }
.spinner { width: 24px; height: 24px; border-radius: 12px; margin-right: 14px;
    border: 3px solid #e4e4de; border-top-color: #407cc5; }
.spinner-text { height: 18px; color: #76766d; font-size: 13px; }

/* ---- Data table ---- */
.table { border: 1px solid #ebebe4; border-radius: 10px; }
.thead { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr;
    background-color: #fafaf7; padding: 12px 16px; border-radius: 10px 10px 0 0; }
.th { height: 16px; color: #a3a399; font-size: 12px; font-weight: 600; }
.th-right { text-align: right; }
.trow { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr; align-items: center;
    padding: 12px 16px; border-top: 1px solid #f0f0e9; }
.td { height: 20px; color: #1b1b18; font-size: 14px; }
.td-name { font-weight: 500; }
.td-muted { color: #76766d; }
.td-mono { color: #1b1b18; }
.td-right { text-align: right; }

/* ---- Feedback ---- */
.feedback-row { display: flex; flex-direction: row; align-items: center; height: 40px; }
"#;

/// The complete **Flat / Corporate SaaS** stylesheet (theme index 3) — cool slate
/// surfaces (`#f1f5f9` page), white cards with a `1px #e2e8f0` border and *no*
/// shadow, medium 10px radius, and one decisive blue accent (`#2563eb`) with a red
/// danger (`#dc2626`). Border-driven structure over depth. Same semantic classes
/// and shared layout numbers as [`MATERIAL_CSS`]; solid fills only, faithful on CPU.
///
/// # Approximated
/// - Mono eyebrow **`letter-spacing`/`text-transform`** dropped (no engine support;
///   labels pre-uppercased in the view).
/// - Card-cover **`repeating-linear-gradient`** → flat slate fill.
const FLAT_CSS: &str = r#"
* { box-sizing: border-box; }
#app { display: flex; flex-direction: column; background-color: #f1f5f9; }

/* ---- Switcher bar ---- */
.kit-switcher { display: flex; flex-direction: row; align-items: center; height: 60px;
    padding-left: 18px; padding-right: 18px; background-color: #ffffff;
    border-bottom: 1px solid #e2e8f0; }
.kit-brand { display: flex; flex-direction: row; align-items: center; width: 60px; height: 60px; }
.kit-logo { width: 11px; height: 11px; border-radius: 4px; background-color: #2563eb; }
.kit-brand-name { width: 40px; height: 16px; margin-left: 8px; color: #0f172a;
    font-size: 13px; font-weight: 700; }
.kit-tabs { display: flex; flex-direction: row; }
.kit-tab { display: flex; flex-direction: column; align-items: center; justify-content: center;
    width: 108px; height: 60px; }
.kit-tab-label { height: 16px; color: #94a3b8; font-size: 12px; font-weight: 600; text-align: center; }
.kit-tab.active .kit-tab-label { color: #0f172a; }
.kit-underline { width: 60px; height: 2px; border-radius: 2px; background-color: #2563eb; margin-top: 6px; }

/* ---- Gallery frame ---- */
.m3-page { display: flex; flex-direction: column; padding: 34px 40px 60px; }
.m3-head { display: flex; flex-direction: column; height: 108px; margin-bottom: 26px; }
.m3-eyebrow { height: 16px; color: #94a3b8; font-size: 11px; font-weight: 600; }
.m3-title { height: 48px; color: #0f172a; font-size: 38px; font-weight: 700; }
.m3-sub { height: 22px; color: #64748b; font-size: 15px; }

.gallery-grid { display: flex; flex-direction: column; }
.row { display: flex; flex-direction: row; align-items: stretch; margin-bottom: 24px; }
.row.r1 { height: 232px; }
.row.r2 { height: 336px; }
.row.r3 { height: 420px; }
.row.r4 { height: 252px; }

/* Cards: white, 1px slate border, medium radius, no shadow. */
.card { background-color: #ffffff; border-radius: 10px; padding: 22px;
    border: 1px solid #e2e8f0; }
.card.col { width: 588px; margin-right: 24px; }
.card.full { width: 1200px; margin-bottom: 24px; }
.card.full.table-c { height: 288px; }
.card.full.feedback-c { height: 132px; }
.card-label { height: 16px; margin-bottom: 16px; color: #94a3b8; font-size: 11px; font-weight: 600; }

/* ---- Buttons: solid blue primary, white+border secondary, 8px radius ---- */
.btn-row { display: flex; flex-direction: row; align-items: center; height: 56px; }
.btn { display: flex; flex-direction: row; align-items: center; justify-content: center;
    height: 40px; border-radius: 8px; padding-left: 20px; padding-right: 20px; margin-right: 10px; }
.btn-label { height: 18px; font-size: 14px; font-weight: 600; text-align: center; }
.btn-filled { width: 110px; background-color: #2563eb; }
.btn-filled .btn-label { color: #ffffff; }
.btn-tonal { width: 104px; background-color: #eff6ff; border: 1px solid #bfdbfe; }
.btn-tonal .btn-label { color: #1d4ed8; }
.btn-outlined { width: 128px; background-color: #ffffff; border: 1px solid #cbd5e1; }
.btn-outlined .btn-label { color: #334155; }
.btn-text { width: 92px; padding-left: 16px; padding-right: 16px; background-color: #ffffff; }
.btn-text .btn-label { color: #2563eb; }
.btn-dialog { width: 152px; }
.btn-elevated { width: 132px; background-color: #ffffff; border: 1px solid #cbd5e1; }
.btn-elevated .btn-label { color: #334155; }
.btn-disabled { width: 128px; background-color: #f1f5f9; border: 1px solid #e2e8f0; }
.btn-disabled .btn-label { color: #cbd5e1; }
/* FAB → the square icon button of the SaaS set. */
.fab { display: flex; flex-direction: row; align-items: center; justify-content: center;
    width: 56px; height: 56px; border-radius: 8px; background-color: #ffffff; border: 1px solid #cbd5e1; }
.fab-glyph { height: 30px; color: #475569; font-size: 26px; font-weight: 400; text-align: center; }
.btn-sm { height: 38px; }
.btn-wide { width: 168px; }

/* ---- Selection ---- */
.sel-cols { display: flex; flex-direction: row; }
.sel-col { display: flex; flex-direction: column; width: 168px; }
.sel-item { display: flex; flex-direction: row; align-items: center; height: 34px; }
.sel-col .sel-item { margin-bottom: 14px; }
.sel-col.wide-gap .sel-item { margin-bottom: 18px; height: 32px; }
.sel-text { height: 20px; margin-left: 11px; color: #0f172a; font-size: 14px; }
.checkbox-on { display: flex; align-items: center; justify-content: center;
    width: 18px; height: 18px; border-radius: 5px; background-color: #2563eb; }
.check-glyph { height: 14px; color: #ffffff; font-size: 12px; text-align: center; }
.checkbox-off { width: 18px; height: 18px; border-radius: 5px; border: 1px solid #cbd5e1; }
.checkbox-disabled { width: 18px; height: 18px; border-radius: 5px; border: 1px solid #e2e8f0;
    background-color: #f1f5f9; }
.radio-on { display: flex; align-items: center; justify-content: center;
    width: 20px; height: 20px; border-radius: 10px; border: 1px solid #cbd5e1; }
.radio-dot { width: 10px; height: 10px; border-radius: 5px; background-color: #2563eb; }
.radio-off { width: 20px; height: 20px; border-radius: 10px; border: 1px solid #cbd5e1; }
.toggle-on { display: flex; flex-direction: row; align-items: center; justify-content: flex-end;
    width: 44px; height: 24px; border-radius: 12px; background-color: #2563eb; padding: 3px; }
.toggle-knob { width: 18px; height: 18px; border-radius: 9px; background-color: #ffffff; }
.toggle-off { display: flex; flex-direction: row; align-items: center; justify-content: flex-start;
    width: 44px; height: 24px; border-radius: 12px; background-color: #cbd5e1; padding: 3px; }
.toggle-knob-off { width: 18px; height: 18px; border-radius: 9px; background-color: #ffffff; }

/* ---- Text fields ---- */
.field-col { display: flex; flex-direction: column; }
.field { margin-bottom: 16px; }
.field-filled { background-color: #ffffff; border-radius: 8px; border: 1px solid #cbd5e1; padding: 9px 12px; }
.field-outlined { background-color: #ffffff; border: 1px solid #cbd5e1; border-radius: 8px; padding: 9px 12px; }
.field-error { background-color: #fef2f2; border: 1px solid #dc2626; border-radius: 8px; padding: 9px 12px; }
.field-label-primary { height: 14px; margin-bottom: 4px; color: #475569; font-size: 11px; font-weight: 500; }
.field-label { height: 14px; margin-bottom: 4px; color: #475569; font-size: 11px; font-weight: 500; }
.field-label-error { height: 14px; margin-bottom: 4px; color: #dc2626; font-size: 11px; font-weight: 500; }
.field-value { height: 20px; color: #0f172a; font-size: 15px; }
.field-placeholder { height: 20px; color: #94a3b8; font-size: 15px; }
.field-error-text { height: 14px; margin-top: 5px; color: #dc2626; font-size: 11px; }

/* ---- Menu / dropdown ---- */
.select-box { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    width: 260px; height: 44px; background-color: #ffffff; border: 1px solid #cbd5e1; border-radius: 8px; padding: 0 14px; }
.select-value { height: 20px; color: #0f172a; font-size: 15px; }
.select-chevron { width: 16px; height: 20px; color: #94a3b8; font-size: 14px; text-align: right; }
.menu-panel { display: flex; flex-direction: column; width: 260px; height: 136px; margin-top: 8px;
    background-color: #ffffff; border: 1px solid #e2e8f0; border-radius: 8px; padding-top: 6px; padding-bottom: 6px;
    box-shadow: 0 10px 28px rgba(15,23,42,0.12); }
.menu-item { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 40px; padding: 0 14px; }
.menu-item.selected { background-color: #f1f5f9; border-radius: 6px; }
.menu-text { height: 20px; color: #0f172a; font-size: 15px; }
.menu-check { width: 16px; height: 20px; color: #2563eb; font-size: 14px; text-align: right; }
.chips-row { display: flex; flex-direction: row; align-items: center; height: 32px; margin-top: 16px; }
.chip { display: flex; flex-direction: row; align-items: center; justify-content: center; height: 32px;
    border-radius: 7px; padding-left: 12px; padding-right: 12px; margin-right: 8px; }
.chip-filled { width: 112px; background-color: #f1f5f9; border: 1px solid #e2e8f0; }
.chip-filled .chip-text { color: #334155; }
.chip-check { height: 16px; margin-right: 6px; color: #64748b; font-size: 12px; }
.chip-outlined { width: 104px; background-color: #f1f5f9; border: 1px solid #e2e8f0; }
.chip-outlined .chip-text { color: #334155; }
.chip-tonal { background-color: #f1f5f9; border: 1px solid #e2e8f0; }
.chip-tonal .chip-text { color: #334155; }
.chip-wide { width: 172px; }
.chip-text { height: 18px; font-size: 14px; font-weight: 600; }

/* ---- Card & list ---- */
.mini-card { display: flex; flex-direction: column; height: 208px; background-color: #ffffff;
    border: 1px solid #e2e8f0; border-radius: 10px; }
.mini-cover { display: flex; align-items: center; justify-content: center; height: 84px;
    border-radius: 10px 10px 0 0; background-color: #e2e8f0; }
.mini-cover-text { height: 16px; color: #94a3b8; font-size: 11px; text-align: center; }
.mini-body { display: flex; flex-direction: column; height: 124px; padding: 16px; }
.mini-title { height: 22px; color: #0f172a; font-size: 16px; font-weight: 700; }
.mini-sub { height: 18px; margin-top: 4px; color: #64748b; font-size: 13px; }
.mini-actions { display: flex; flex-direction: row; align-items: center; height: 38px; margin-top: 14px; }
.people { display: flex; flex-direction: column; height: 100px; margin-top: 14px; }
.person { display: flex; flex-direction: row; align-items: center; height: 50px; }
.person-meta { display: flex; flex-direction: column; justify-content: center; height: 50px;
    margin-left: 13px; width: 420px; }
.person-name { height: 18px; color: #0f172a; font-size: 14px; }
.person-email { height: 16px; margin-top: 2px; color: #64748b; font-size: 12px; }
.person-menu { width: 24px; height: 20px; color: #94a3b8; font-size: 18px; text-align: right; }
.avatar { display: flex; align-items: center; justify-content: center;
    width: 36px; height: 36px; border-radius: 9px; }
.avatar-text { height: 16px; font-size: 13px; font-weight: 700; text-align: center; color: #ffffff; }
.avatar-ak { background-color: #2563eb; }
.avatar-mr { background-color: #0891b2; }
.avatar-jp { background-color: #7c3aed; }
.avatar-more { background-color: #f1f5f9; }
.avatar-more .avatar-text { color: #64748b; }

/* ---- Navigation ---- */
.breadcrumb { display: flex; flex-direction: row; align-items: center; height: 20px; margin-bottom: 18px; }
.crumb { height: 18px; margin-right: 8px; color: #64748b; font-size: 13px; }
.crumb-sep { height: 18px; margin-right: 8px; color: #cbd5e1; font-size: 13px; }
.crumb-active { height: 18px; color: #0f172a; font-size: 13px; font-weight: 600; }
.nav-tabs { display: flex; flex-direction: row; height: 42px;
    border-bottom: 2px solid #e2e8f0; margin-bottom: 14px; }
.nav-tab { display: flex; flex-direction: column; align-items: center; justify-content: flex-end;
    width: 96px; }
.nav-tab-label { height: 24px; color: #64748b; font-size: 14px; font-weight: 600; text-align: center; }
.nav-tab-label.active { color: #2563eb; }
.nav-tab-underline { width: 96px; height: 2px; background-color: #2563eb; }
.nav-list { display: flex; flex-direction: column; width: 260px; }
.nav-item { display: flex; flex-direction: row; align-items: center; height: 40px;
    border-radius: 7px; padding-left: 12px; padding-right: 12px; margin-bottom: 4px; }
.nav-item.active { background-color: #eff6ff; border-left: 3px solid #2563eb; }
.nav-item-text { height: 20px; margin-left: 10px; color: #475569; font-size: 14px; }
.nav-item.active .nav-item-text { color: #1d4ed8; font-weight: 600; }
.dot-on { width: 6px; height: 6px; border-radius: 2px; background-color: #2563eb; }
.dot-off { width: 6px; height: 6px; border-radius: 2px; background-color: #cbd5e1; }

/* ---- Badges / chips / avatars: solid pastel status pills, 6px radius ---- */
.badge-row { display: flex; flex-direction: row; align-items: center; height: 26px; margin-bottom: 22px; }
.badge { display: flex; align-items: center; justify-content: center; height: 24px;
    border-radius: 6px; margin-right: 8px; }
.badge-text { height: 16px; font-size: 12px; font-weight: 600; text-align: center; }
.badge-success { width: 76px; background-color: #dcfce7; }
.badge-success .badge-text { color: #166534; }
.badge-warn { width: 84px; background-color: #fef3c7; }
.badge-warn .badge-text { color: #92400e; }
.badge-error { width: 66px; background-color: #fee2e2; }
.badge-error .badge-text { color: #991b1b; }
.badge-neutral { width: 64px; background-color: #f1f5f9; }
.badge-neutral .badge-text { color: #64748b; }
.badge-count { display: flex; align-items: center; justify-content: center;
    width: 24px; height: 20px; border-radius: 6px; background-color: #2563eb; }
.badge-count-text { height: 14px; color: #ffffff; font-size: 11px; font-weight: 700; text-align: center; }
.avatar-stack { display: flex; flex-direction: row; align-items: center; height: 40px; }
.avatar.stack { border: 2px solid #ffffff; }
.avatar.overlap { margin-left: -9px; }

/* ---- Sliders / progress ---- */
.slider-head { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 18px; margin-bottom: 10px; }
.slider-name { height: 16px; color: #64748b; font-size: 13px; }
.slider-val { height: 16px; color: #2563eb; font-size: 13px; font-weight: 600; text-align: right; }
.slider { position: relative; height: 24px; margin-bottom: 24px; }
.slider-track { position: absolute; left: 0; right: 0; top: 9px; height: 6px;
    border-radius: 3px; background-color: #e2e8f0; }
.slider-fill { position: absolute; left: 0; top: 9px; width: 55%; height: 6px;
    border-radius: 3px; background-color: #2563eb; }
.slider-knob { position: absolute; left: 53%; top: 3px; width: 18px; height: 18px;
    border-radius: 9px; background-color: #ffffff; border: 2px solid #2563eb; }
.progress-label { height: 18px; margin-bottom: 8px; color: #64748b; font-size: 13px; }
.progress { height: 8px; border-radius: 4px; background-color: #e2e8f0; margin-bottom: 20px; }
.progress-fill { width: 72%; height: 8px; border-radius: 4px; background-color: #2563eb; }
.spinner-row { display: flex; flex-direction: row; align-items: center; height: 28px; }
.spinner { width: 24px; height: 24px; border-radius: 12px; margin-right: 14px;
    border: 3px solid #e2e8f0; border-top-color: #2563eb; }
.spinner-text { height: 18px; color: #64748b; font-size: 13px; }

/* ---- Data table ---- */
.table { border: 1px solid #e2e8f0; border-radius: 8px; }
.thead { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr;
    background-color: #f8fafc; padding: 12px 16px; border-radius: 8px 8px 0 0; }
.th { height: 16px; color: #94a3b8; font-size: 12px; font-weight: 700; }
.th-right { text-align: right; }
.trow { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr; align-items: center;
    padding: 12px 16px; border-top: 1px solid #f1f5f9; }
.td { height: 20px; color: #0f172a; font-size: 14px; }
.td-name { font-weight: 600; }
.td-muted { color: #64748b; }
.td-mono { color: #0f172a; }
.td-right { text-align: right; }

/* ---- Feedback ---- */
.feedback-row { display: flex; flex-direction: row; align-items: center; height: 40px; }
"#;

/// The complete **Metro / Modern UI** stylesheet (theme index 5) — the Windows 8
/// "Modern UI" look: white page, flat solid-color **tiles** with **sharp zero-radius
/// corners**, a bold blue accent (`#2d89ef`) plus orange/green/red/purple accents,
/// thin type, and a dark table header. The signature is the *absence* of rounding:
/// every surface is a hard rectangle. Same semantic classes and shared layout
/// numbers as [`MATERIAL_CSS`]; solid fills only, faithful on CPU.
///
/// # Metro-defining choices against the shared view
/// - **`border-radius: 0` everywhere** except the radio (the one intentionally round
///   control) and the circular spinner ring — matching the spec.
/// - Cards become flat `#f2f2f2` tiles with **no border and no shadow**.
/// - Toggles are **rectangular** (a bordered box + a sliding rectangular knob), not
///   pills — Metro's on/off switch.
/// - Slider thumb is a tall thin **bar**, badges/avatars are hard color blocks, the
///   table header is a solid **dark** band.
///
/// # Approximated
/// - Mono eyebrow **`letter-spacing`/`text-transform`** dropped (labels pre-cased).
/// - The card-cover live-tile hatch → a solid accent tile fill.
const METRO_CSS: &str = r#"
* { box-sizing: border-box; }
#app { display: flex; flex-direction: column; background-color: #ffffff; }

/* ---- Switcher bar ---- */
.kit-switcher { display: flex; flex-direction: row; align-items: center; height: 60px;
    padding-left: 18px; padding-right: 18px; background-color: #1d1d1d; }
.kit-brand { display: flex; flex-direction: row; align-items: center; width: 60px; height: 60px; }
.kit-logo { width: 11px; height: 11px; background-color: #2d89ef; }
.kit-brand-name { width: 40px; height: 16px; margin-left: 8px; color: #ffffff;
    font-size: 13px; font-weight: 600; }
.kit-tabs { display: flex; flex-direction: row; }
.kit-tab { display: flex; flex-direction: column; align-items: center; justify-content: center;
    width: 108px; height: 60px; }
.kit-tab-label { height: 16px; color: #9a9a9a; font-size: 12px; font-weight: 400; text-align: center; }
.kit-tab.active .kit-tab-label { color: #ffffff; }
.kit-underline { width: 60px; height: 3px; background-color: #2d89ef; margin-top: 6px; }

/* ---- Gallery frame ---- */
.m3-page { display: flex; flex-direction: column; padding: 34px 40px 60px; }
.m3-head { display: flex; flex-direction: column; height: 108px; margin-bottom: 26px; }
.m3-eyebrow { height: 16px; color: #9a9a9a; font-size: 11px; font-weight: 400; }
.m3-title { height: 48px; color: #1d1d1d; font-size: 40px; font-weight: 300; }
.m3-sub { height: 22px; color: #6a6a6a; font-size: 15px; }

.gallery-grid { display: flex; flex-direction: column; }
.row { display: flex; flex-direction: row; align-items: stretch; margin-bottom: 20px; }
.row.r1 { height: 232px; }
.row.r2 { height: 336px; }
.row.r3 { height: 420px; }
.row.r4 { height: 252px; }

/* Cards: flat gray tiles — no border, no shadow, ZERO radius. */
.card { background-color: #f2f2f2; border-radius: 0; padding: 22px; }
.card.col { width: 590px; margin-right: 20px; }
.card.full { width: 1200px; margin-bottom: 20px; }
.card.full.table-c { height: 288px; }
.card.full.feedback-c { height: 132px; }
.card-label { height: 16px; margin-bottom: 16px; color: #9a9a9a; font-size: 11px; font-weight: 400; }

/* ---- Buttons: flat sharp blocks ---- */
.btn-row { display: flex; flex-direction: row; align-items: center; height: 56px; }
.btn { display: flex; flex-direction: row; align-items: center; justify-content: center;
    height: 40px; border-radius: 0; padding-left: 20px; padding-right: 20px; margin-right: 10px; }
.btn-label { height: 18px; font-size: 14px; font-weight: 600; text-align: center; }
.btn-filled { width: 110px; background-color: #2d89ef; }
.btn-filled .btn-label { color: #ffffff; }
.btn-tonal { width: 104px; background-color: #ffffff; border: 2px solid #1d1d1d; }
.btn-tonal .btn-label { color: #1d1d1d; }
.btn-outlined { width: 128px; background-color: #ffffff; border: 2px solid #1d1d1d; }
.btn-outlined .btn-label { color: #1d1d1d; }
.btn-text { width: 92px; padding-left: 16px; padding-right: 16px; background-color: #f2f2f2; }
.btn-text .btn-label { color: #2d89ef; }
.btn-dialog { width: 152px; }
.btn-elevated { width: 132px; background-color: #da532c; }
.btn-elevated .btn-label { color: #ffffff; }
.btn-disabled { width: 128px; background-color: #d6d6d6; }
.btn-disabled .btn-label { color: #9a9a9a; }
/* FAB → a flat square accent tile with the + glyph. */
.fab { display: flex; flex-direction: row; align-items: center; justify-content: center;
    width: 56px; height: 56px; border-radius: 0; background-color: #ffffff; border: 2px solid #d0d0d0; }
.fab-glyph { height: 30px; color: #2d89ef; font-size: 26px; font-weight: 400; text-align: center; }
.btn-sm { height: 38px; }
.btn-wide { width: 168px; }

/* ---- Selection: sharp square checks, round radios, rectangular toggles ---- */
.sel-cols { display: flex; flex-direction: row; }
.sel-col { display: flex; flex-direction: column; width: 168px; }
.sel-item { display: flex; flex-direction: row; align-items: center; height: 34px; }
.sel-col .sel-item { margin-bottom: 14px; }
.sel-col.wide-gap .sel-item { margin-bottom: 18px; height: 32px; }
.sel-text { height: 20px; margin-left: 11px; color: #1d1d1d; font-size: 14px; }
.checkbox-on { display: flex; align-items: center; justify-content: center;
    width: 20px; height: 20px; border-radius: 0; background-color: #2d89ef; }
.check-glyph { height: 14px; color: #ffffff; font-size: 12px; text-align: center; }
.checkbox-off { width: 20px; height: 20px; border-radius: 0; background-color: #ffffff; border: 2px solid #5a5a5a; }
.checkbox-disabled { width: 20px; height: 20px; border-radius: 0; background-color: #e6e6e6; border: 2px solid #cfcfcf; }
/* Radio is the one deliberately round Metro control. */
.radio-on { display: flex; align-items: center; justify-content: center;
    width: 20px; height: 20px; border-radius: 10px; background-color: #ffffff; border: 2px solid #5a5a5a; }
.radio-dot { width: 10px; height: 10px; border-radius: 5px; background-color: #2d89ef; }
.radio-off { width: 20px; height: 20px; border-radius: 10px; background-color: #ffffff; border: 2px solid #5a5a5a; }
/* Toggle: a rectangular bordered box with a sliding rectangular knob. */
.toggle-on { display: flex; flex-direction: row; align-items: center; justify-content: flex-end;
    width: 46px; height: 22px; border-radius: 0; background-color: #2d89ef; border: 1px solid #2d89ef; padding: 2px; }
.toggle-knob { width: 14px; height: 16px; border-radius: 0; background-color: #ffffff; }
.toggle-off { display: flex; flex-direction: row; align-items: center; justify-content: flex-start;
    width: 46px; height: 22px; border-radius: 0; background-color: #ffffff; border: 1px solid #5a5a5a; padding: 2px; }
.toggle-knob-off { width: 14px; height: 16px; border-radius: 0; background-color: #5a5a5a; }

/* ---- Text fields: sharp white boxes, 2px gray border ---- */
.field-col { display: flex; flex-direction: column; }
.field { margin-bottom: 16px; }
.field-filled { background-color: #ffffff; border-radius: 0; border: 2px solid #d0d0d0; padding: 8px 12px; }
.field-outlined { background-color: #ffffff; border: 2px solid #d0d0d0; border-radius: 0; padding: 8px 12px; }
.field-error { background-color: #ffffff; border: 2px solid #da532c; border-radius: 0; padding: 8px 12px; }
.field-label-primary { height: 14px; margin-bottom: 4px; color: #6a6a6a; font-size: 11px; }
.field-label { height: 14px; margin-bottom: 4px; color: #6a6a6a; font-size: 11px; }
.field-label-error { height: 14px; margin-bottom: 4px; color: #da532c; font-size: 11px; }
.field-value { height: 20px; color: #1d1d1d; font-size: 15px; }
.field-placeholder { height: 20px; color: #9a9a9a; font-size: 15px; }
.field-error-text { height: 14px; margin-top: 5px; color: #da532c; font-size: 11px; }

/* ---- Menu / dropdown ---- */
.select-box { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    width: 260px; height: 44px; background-color: #ffffff; border: 2px solid #d0d0d0; border-radius: 0; padding: 0 12px; }
.select-value { height: 20px; color: #1d1d1d; font-size: 15px; }
.select-chevron { width: 16px; height: 20px; color: #6a6a6a; font-size: 14px; text-align: right; }
.menu-panel { display: flex; flex-direction: column; width: 260px; height: 136px; margin-top: 6px;
    background-color: #ffffff; border: 2px solid #2d89ef; border-radius: 0; }
.menu-item { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 44px; padding: 0 12px; }
.menu-item.selected { background-color: #2d89ef; }
.menu-text { height: 20px; color: #1d1d1d; font-size: 15px; }
.menu-item.selected .menu-text { color: #ffffff; }
.menu-check { width: 16px; height: 20px; color: #ffffff; font-size: 14px; text-align: right; }
.chips-row { display: flex; flex-direction: row; align-items: center; height: 32px; margin-top: 16px; }
.chip { display: flex; flex-direction: row; align-items: center; justify-content: center; height: 32px;
    border-radius: 0; padding-left: 12px; padding-right: 12px; margin-right: 8px; }
.chip-filled { width: 112px; background-color: #dcdcdc; }
.chip-filled .chip-text { color: #1d1d1d; }
.chip-check { height: 16px; margin-right: 6px; color: #6a6a6a; font-size: 12px; }
.chip-outlined { width: 104px; background-color: #dcdcdc; }
.chip-outlined .chip-text { color: #1d1d1d; }
.chip-tonal { background-color: #dcdcdc; }
.chip-tonal .chip-text { color: #1d1d1d; }
.chip-wide { width: 172px; }
.chip-text { height: 18px; font-size: 14px; font-weight: 600; }

/* ---- Card & list: solid accent live-tile + hard-block avatars ---- */
.mini-card { display: flex; flex-direction: column; height: 208px; background-color: #ffffff; border-radius: 0; }
.mini-cover { display: flex; align-items: center; justify-content: center; height: 96px;
    border-radius: 0; background-color: #2d89ef; }
.mini-cover-text { height: 16px; color: #ffffff; font-size: 13px; font-weight: 600; text-align: center; }
.mini-body { display: flex; flex-direction: column; height: 112px; padding: 14px 2px; }
.mini-title { height: 22px; color: #1d1d1d; font-size: 16px; font-weight: 600; }
.mini-sub { height: 18px; margin-top: 4px; color: #6a6a6a; font-size: 13px; }
.mini-actions { display: flex; flex-direction: row; align-items: center; height: 38px; margin-top: 14px; }
.people { display: flex; flex-direction: column; height: 100px; margin-top: 14px; }
.person { display: flex; flex-direction: row; align-items: center; height: 50px; }
.person-meta { display: flex; flex-direction: column; justify-content: center; height: 50px;
    margin-left: 13px; width: 420px; }
.person-name { height: 18px; color: #1d1d1d; font-size: 14px; }
.person-email { height: 16px; margin-top: 2px; color: #6a6a6a; font-size: 12px; }
.person-menu { width: 24px; height: 20px; color: #9a9a9a; font-size: 18px; text-align: right; }
.avatar { display: flex; align-items: center; justify-content: center;
    width: 34px; height: 34px; border-radius: 0; }
.avatar-text { height: 16px; font-size: 13px; font-weight: 600; text-align: center; color: #ffffff; }
.avatar-ak { background-color: #da532c; }
.avatar-mr { background-color: #00a300; }
.avatar-jp { background-color: #603cba; }
.avatar-more { background-color: #dcdcdc; }
.avatar-more .avatar-text { color: #6a6a6a; }

/* ---- Navigation ---- */
.breadcrumb { display: flex; flex-direction: row; align-items: center; height: 20px; margin-bottom: 18px; }
.crumb { height: 18px; margin-right: 8px; color: #6a6a6a; font-size: 13px; }
.crumb-sep { height: 18px; margin-right: 8px; color: #bdbdbd; font-size: 13px; }
.crumb-active { height: 18px; color: #1d1d1d; font-size: 13px; font-weight: 600; }
.nav-tabs { display: flex; flex-direction: row; height: 42px; margin-bottom: 14px; }
.nav-tab { display: flex; flex-direction: column; align-items: center; justify-content: flex-end;
    width: 96px; }
.nav-tab-label { height: 30px; color: #9a9a9a; font-size: 20px; font-weight: 300; text-align: center; }
.nav-tab-label.active { color: #1d1d1d; }
.nav-tab-underline { width: 0; height: 0; }
.nav-list { display: flex; flex-direction: column; width: 260px; }
.nav-item { display: flex; flex-direction: row; align-items: center; height: 40px;
    border-radius: 0; padding-left: 12px; padding-right: 12px; }
.nav-item.active { background-color: #2d89ef; }
.nav-item-text { height: 20px; margin-left: 10px; color: #5a5a5a; font-size: 14px; }
.nav-item.active .nav-item-text { color: #ffffff; font-weight: 600; }
.dot-on { width: 6px; height: 6px; border-radius: 0; background-color: #ffffff; }
.dot-off { width: 6px; height: 6px; border-radius: 0; background-color: #bdbdbd; }

/* ---- Badges / chips / avatars: solid hard color blocks ---- */
.badge-row { display: flex; flex-direction: row; align-items: center; height: 26px; margin-bottom: 22px; }
.badge { display: flex; align-items: center; justify-content: center; height: 24px;
    border-radius: 0; margin-right: 8px; }
.badge-text { height: 16px; font-size: 12px; font-weight: 600; text-align: center; color: #ffffff; }
.badge-success { width: 76px; background-color: #00a300; }
.badge-warn { width: 84px; background-color: #fa6800; }
.badge-error { width: 66px; background-color: #da532c; }
.badge-neutral { width: 64px; background-color: #dcdcdc; }
.badge-neutral .badge-text { color: #5a5a5a; }
.badge-count { display: flex; align-items: center; justify-content: center;
    width: 24px; height: 20px; border-radius: 0; background-color: #1d1d1d; }
.badge-count-text { height: 14px; color: #ffffff; font-size: 11px; font-weight: 700; text-align: center; }
.avatar-stack { display: flex; flex-direction: row; align-items: center; height: 40px; }
.avatar.stack { border: 2px solid #ffffff; }
.avatar.overlap { margin-left: 6px; }

/* ---- Sliders / progress: thin bar thumb, flat track ---- */
.slider-head { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 18px; margin-bottom: 10px; }
.slider-name { height: 16px; color: #6a6a6a; font-size: 13px; }
.slider-val { height: 16px; color: #2d89ef; font-size: 13px; font-weight: 600; text-align: right; }
.slider { position: relative; height: 24px; margin-bottom: 24px; }
.slider-track { position: absolute; left: 0; right: 0; top: 10px; height: 4px;
    border-radius: 0; background-color: #cfcfcf; }
.slider-fill { position: absolute; left: 0; top: 10px; width: 55%; height: 4px;
    border-radius: 0; background-color: #2d89ef; }
.slider-knob { position: absolute; left: 54%; top: 1px; width: 8px; height: 22px;
    border-radius: 0; background-color: #2d89ef; }
.progress-label { height: 18px; margin-bottom: 8px; color: #6a6a6a; font-size: 13px; }
.progress { height: 6px; border-radius: 0; background-color: #cfcfcf; margin-bottom: 20px; }
.progress-fill { width: 72%; height: 6px; border-radius: 0; background-color: #2d89ef; }
.spinner-row { display: flex; flex-direction: row; align-items: center; height: 28px; }
.spinner { width: 24px; height: 24px; border-radius: 12px; margin-right: 14px;
    border: 3px solid #cfcfcf; border-top-color: #2d89ef; }
.spinner-text { height: 18px; color: #6a6a6a; font-size: 13px; }

/* ---- Data table: dark header band, ruled rows, no radius ---- */
.table { border-radius: 0; background-color: #ffffff; }
.thead { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr;
    background-color: #1d1d1d; padding: 12px 16px; border-radius: 0; }
.th { height: 16px; color: #ffffff; font-size: 12px; font-weight: 600; }
.th-right { text-align: right; }
.trow { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr; align-items: center;
    padding: 12px 16px; border-bottom: 1px solid #e6e6e6; }
.td { height: 20px; color: #1d1d1d; font-size: 14px; }
.td-name { font-weight: 600; }
.td-muted { color: #6a6a6a; }
.td-mono { color: #1d1d1d; }
.td-right { text-align: right; }

/* ---- Feedback ---- */
.feedback-row { display: flex; flex-direction: row; align-items: center; height: 40px; }
"#;

/// The **Skeuomorphic** stylesheet (theme index 4) — the gradient/shadow-heavy
/// "glossy plastic + brushed beige" look, authored against the same semantic
/// classes as [`MATERIAL_CSS`]. The view is untouched; only these tokens change.
///
/// # Techniques (the ones that make it skeuomorphic)
///
/// - **2-stop vertical `linear-gradient` faces.** Every panel/button gets a
///   top-light → bottom-dark ramp (`linear-gradient(#light,#dark)`), which on the
///   Vello backend paints a real smooth ramp — the source of the "molded plastic"
///   sheen. On the CPU backend the same gradient degrades to its FIRST stop (a
///   flat light fill) per the effect-split policy, so the CPU PNG looks flat by
///   design — that contrast is the whole point of the two renders.
/// - **`inset` gloss highlight + `outset` drop shadow in ONE `box-shadow` list.**
///   The engine honors a comma list of shadows and paints drop shadows BEHIND the
///   background and inset shadows ON TOP of it, so a single declaration like
///   `box-shadow: inset 0 1px 0 rgba(255,255,255,.8), 0 2px 5px rgba(0,0,0,.25)`
///   gives both the raised top-edge highlight and the cast shadow. Both blur
///   through Vello's real gaussian; the CPU backend approximates them as flat
///   offset rects.
/// - **1px bevel borders** (`1px solid #a9a496`) around the beige faces, and
///   **recessed wells** for inputs / slider tracks (an `inset` shadow with no
///   drop shadow reads as "sunken").
/// - **Radius ~10–12px** throughout for the rounded-hardware feel.
///
/// # Approximated / omitted (documented per project honesty policy)
///
/// - **`text-shadow` emboss** — the imported design embosses every label with
///   `text-shadow: 0 1px 0 rgba(255,255,255,.x)`. We have no `text-shadow`
///   property, so it is **omitted**; text renders crisp without the engraved
///   highlight. This is the one signature we cannot reproduce.
/// - **>2-stop / `repeating-linear-gradient` gloss** (a hard specular band across
///   the top third) is approximated by a plain 2-stop ramp — the engine models a
///   single `linear-gradient` with auto-distributed stops, not a repeating one.
/// - **Per-corner / multi-layer inset bevels** collapse to one `inset` shadow with
///   uniform corner radii (`draw_shadow` blurs a uniformly-rounded rect).
const SKEUOMORPHIC_CSS: &str = r#"
* { box-sizing: border-box; }
/* Page: the brushed-beige backdrop gradient (real ramp on Vello, first stop on CPU). */
#app { display: flex; flex-direction: column; background-image: linear-gradient(#cfcabb, #bdb8a8); }

/* ---- Switcher bar: a dark brushed-metal strip with a top gloss line ---- */
.kit-switcher { display: flex; flex-direction: row; align-items: center; height: 60px;
    padding-left: 18px; padding-right: 18px;
    background-image: linear-gradient(#5a5a52, #33332e);
    border-bottom: 1px solid #22221e;
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.18), 0 2px 4px rgba(0,0,0,0.35); }
.kit-brand { display: flex; flex-direction: row; align-items: center; width: 60px; height: 60px; }
.kit-logo { width: 11px; height: 11px; border-radius: 3px;
    background-image: linear-gradient(#f5cd6b, #e0a82f); border: 1px solid #b8860b;
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.6); }
.kit-brand-name { width: 40px; height: 16px; margin-left: 8px; color: #f2efe8;
    font-size: 13px; font-weight: 700; }
.kit-tabs { display: flex; flex-direction: row; }
.kit-tab { display: flex; flex-direction: column; align-items: center; justify-content: center;
    width: 108px; height: 60px; }
.kit-tab-label { height: 16px; color: #b8b3a4; font-size: 12px; font-weight: 500; text-align: center; }
.kit-tab.active .kit-tab-label { color: #ffffff; }
.kit-underline { width: 60px; height: 3px; border-radius: 2px; margin-top: 6px;
    background-image: linear-gradient(#7eb4f2, #3a78db);
    box-shadow: 0 1px 2px rgba(0,0,0,0.4); }

/* ---- Gallery frame ---- */
.m3-page { display: flex; flex-direction: column; padding: 34px 40px 60px; }
.m3-head { display: flex; flex-direction: column; height: 108px; margin-bottom: 26px; }
.m3-eyebrow { height: 16px; color: #8a8474; font-size: 11px; font-weight: 700; }
.m3-title { height: 48px; color: #3a362e; font-size: 38px; font-weight: 700; }
.m3-sub { height: 22px; color: #6b6658; font-size: 15px; }

.gallery-grid { display: flex; flex-direction: column; }
.row { display: flex; flex-direction: row; align-items: stretch; margin-bottom: 24px; }
.row.r1 { height: 232px; }
.row.r2 { height: 336px; }
.row.r3 { height: 420px; }
.row.r4 { height: 252px; }

/* Cards: the molded beige panel — top-light face, bevel border, top gloss line + cast shadow. */
.card { padding: 22px; border-radius: 12px; border: 1px solid #a9a496;
    background-image: linear-gradient(#f2efe8, #e0dccf);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.8), 0 2px 5px rgba(0,0,0,0.25); }
.card.col { width: 588px; margin-right: 24px; }
.card.full { width: 1200px; margin-bottom: 24px; }
.card.full.table-c { height: 288px; }
.card.full.feedback-c { height: 132px; }
.card-label { height: 16px; margin-bottom: 16px; color: #8a8474; font-size: 11px; font-weight: 700; }

/* ---- Buttons: glossy hardware keys ---- */
.btn-row { display: flex; flex-direction: row; align-items: center; height: 56px; }
.btn { display: flex; flex-direction: row; align-items: center; justify-content: center;
    height: 40px; border-radius: 10px; padding-left: 22px; padding-right: 22px; margin-right: 10px;
    border: 1px solid #a9a496;
    background-image: linear-gradient(#fbfbfa, #e6e3da);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.9), 0 1px 2px rgba(0,0,0,0.25); }
.btn-label { height: 18px; font-size: 14px; font-weight: 700; text-align: center; color: #4a4636; }
/* Primary blue: glossy candy button with a darker bevel + white gloss line. */
.btn-filled { width: 110px; border: 1px solid #2560bf;
    background-image: linear-gradient(#6ba8ef, #2f6fd6);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.45), 0 1px 3px rgba(0,0,0,0.35); }
.btn-filled .btn-label { color: #ffffff; }
.btn-tonal { width: 104px; border: 1px solid #9bb7dd;
    background-image: linear-gradient(#dbe7f8, #b9cdec); }
.btn-tonal .btn-label { color: #234a86; }
.btn-outlined { width: 128px; border: 1px solid #a9a496;
    background-image: linear-gradient(#f2efe8, #e0dccf); }
.btn-outlined .btn-label { color: #4a4636; }
.btn-text { width: 92px; padding-left: 16px; padding-right: 16px;
    border: 1px solid transparent; background-image: linear-gradient(#efece4, #efece4);
    box-shadow: none; }
.btn-text .btn-label { color: #2f6fd6; }
.btn-dialog { width: 152px; }
/* Elevated: same glossy face, a heavier cast shadow to float it. */
.btn-elevated { width: 132px; border: 1px solid #a9a496;
    background-image: linear-gradient(#fbfbfa, #e6e3da);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.9), 0 3px 6px rgba(0,0,0,0.3); }
.btn-elevated .btn-label { color: #4a4636; }
/* Disabled: flat, no gloss, no shadow — reads as inert. */
.btn-disabled { width: 128px; border: 1px solid #c7c3b6;
    background-image: linear-gradient(#e4e1d8, #e4e1d8); box-shadow: none; }
.btn-disabled .btn-label { color: #a7a394; }
/* FAB: a glossy blue puck. */
.fab { display: flex; flex-direction: row; align-items: center; justify-content: center;
    width: 56px; height: 56px; border-radius: 16px; border: 1px solid #2560bf;
    background-image: linear-gradient(#6ba8ef, #2f6fd6);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.45), 0 3px 6px rgba(0,0,0,0.35); }
.fab-glyph { height: 30px; color: #ffffff; font-size: 26px; font-weight: 400; text-align: center; }
.btn-sm { height: 38px; }
.btn-wide { width: 168px; }

/* ---- Selection: glossy checks / radios / a recessed toggle groove ---- */
.sel-cols { display: flex; flex-direction: row; }
.sel-col { display: flex; flex-direction: column; width: 168px; }
.sel-item { display: flex; flex-direction: row; align-items: center; height: 34px; }
.sel-col .sel-item { margin-bottom: 14px; }
.sel-col.wide-gap .sel-item { margin-bottom: 18px; height: 32px; }
.sel-text { height: 20px; margin-left: 11px; color: #4a4636; font-size: 14px; }
.checkbox-on { display: flex; align-items: center; justify-content: center;
    width: 18px; height: 18px; border-radius: 4px; border: 1px solid #2560bf;
    background-image: linear-gradient(#6ba8ef, #2f6fd6);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.4), 0 1px 1px rgba(0,0,0,0.3); }
.check-glyph { height: 14px; color: #ffffff; font-size: 12px; text-align: center; }
.checkbox-off { width: 18px; height: 18px; border-radius: 4px; border: 1px solid #a9a496;
    background-image: linear-gradient(#e3e0d8, #fcfbf8);
    box-shadow: inset 0 1px 2px rgba(0,0,0,0.25); }
.checkbox-disabled { width: 18px; height: 18px; border-radius: 4px; border: 1px solid #c7c3b6;
    background-image: linear-gradient(#e4e1d8, #e4e1d8); }
.radio-on { display: flex; align-items: center; justify-content: center;
    width: 20px; height: 20px; border-radius: 10px; border: 1px solid #a9a496;
    background-image: linear-gradient(#e3e0d8, #fcfbf8);
    box-shadow: inset 0 1px 2px rgba(0,0,0,0.2); }
.radio-dot { width: 10px; height: 10px; border-radius: 5px; border: 1px solid #2560bf;
    background-image: linear-gradient(#6ba8ef, #2f6fd6);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.5); }
.radio-off { width: 20px; height: 20px; border-radius: 10px; border: 1px solid #a9a496;
    background-image: linear-gradient(#e3e0d8, #fcfbf8);
    box-shadow: inset 0 1px 2px rgba(0,0,0,0.2); }
/* Toggle: a sunken groove (inset well) with a raised glossy knob. */
.toggle-on { display: flex; flex-direction: row; align-items: center; justify-content: flex-end;
    width: 52px; height: 32px; border-radius: 16px; padding: 4px; border: 1px solid #2560bf;
    background-image: linear-gradient(#3a78db, #6ba8ef);
    box-shadow: inset 0 2px 3px rgba(0,0,0,0.35); }
.toggle-knob { width: 24px; height: 24px; border-radius: 12px; border: 1px solid #b8b3a4;
    background-image: linear-gradient(#ffffff, #dedacf);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.9), 0 1px 2px rgba(0,0,0,0.4); }
.toggle-off { display: flex; flex-direction: row; align-items: center; justify-content: flex-start;
    width: 52px; height: 32px; border-radius: 16px; padding: 6px; border: 1px solid #a9a496;
    background-image: linear-gradient(#c9c5bb, #ddd9d0);
    box-shadow: inset 0 2px 3px rgba(0,0,0,0.3); }
.toggle-knob-off { width: 16px; height: 16px; border-radius: 8px; border: 1px solid #b8b3a4;
    background-image: linear-gradient(#ffffff, #dedacf);
    box-shadow: 0 1px 2px rgba(0,0,0,0.35); }

/* ---- Text fields: recessed wells (inset shadow, light face, bevel border) ---- */
.field-col { display: flex; flex-direction: column; }
.field { margin-bottom: 16px; }
.field-filled { border-radius: 8px; padding: 8px 14px 6px; border: 1px solid #a9a496;
    background-image: linear-gradient(#e3e0d8, #fcfbf8);
    box-shadow: inset 0 2px 3px rgba(0,0,0,0.22); }
.field-outlined { border-radius: 8px; padding: 10px 14px; border: 1px solid #a9a496;
    background-image: linear-gradient(#e3e0d8, #fcfbf8);
    box-shadow: inset 0 2px 3px rgba(0,0,0,0.18); }
.field-error { border-radius: 8px; padding: 10px 14px; border: 1px solid #c0392b;
    background-image: linear-gradient(#f6e2de, #fcf7f6);
    box-shadow: inset 0 2px 3px rgba(0,0,0,0.18); }
.field-label-primary { height: 14px; margin-bottom: 4px; color: #2f6fd6; font-size: 11px; font-weight: 700; }
.field-label { height: 14px; margin-bottom: 4px; color: #6b6658; font-size: 11px; font-weight: 700; }
.field-label-error { height: 14px; margin-bottom: 4px; color: #c0392b; font-size: 11px; font-weight: 700; }
.field-value { height: 20px; color: #3a362e; font-size: 15px; }
.field-placeholder { height: 20px; color: #8a8474; font-size: 15px; }
.field-error-text { height: 14px; margin-top: 5px; color: #c0392b; font-size: 11px; }

/* ---- Menu / dropdown ---- */
.select-box { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    width: 260px; height: 44px; border-radius: 8px; padding: 0 14px; border: 1px solid #a9a496;
    background-image: linear-gradient(#fbfbfa, #e6e3da);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.9), 0 1px 2px rgba(0,0,0,0.22); }
.select-value { height: 20px; color: #3a362e; font-size: 15px; }
.select-chevron { width: 16px; height: 20px; color: #6b6658; font-size: 14px; text-align: right; }
.menu-panel { display: flex; flex-direction: column; width: 260px; height: 136px; margin-top: 8px;
    border-radius: 10px; padding-top: 8px; padding-bottom: 8px; border: 1px solid #a9a496;
    background-image: linear-gradient(#f6f4ee, #e4e0d5);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.8), 0 6px 14px rgba(0,0,0,0.3); }
.menu-item { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 40px; padding: 0 16px; }
.menu-item.selected { border-radius: 6px;
    background-image: linear-gradient(#6ba8ef, #2f6fd6);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.35); }
.menu-text { height: 20px; color: #3a362e; font-size: 15px; }
.menu-item.selected .menu-text { color: #ffffff; }
.menu-check { width: 16px; height: 20px; color: #ffffff; font-size: 14px; text-align: right; }
.chips-row { display: flex; flex-direction: row; align-items: center; height: 32px; margin-top: 16px; }
.chip { display: flex; flex-direction: row; align-items: center; justify-content: center; height: 32px;
    border-radius: 16px; padding-left: 14px; padding-right: 14px; margin-right: 8px;
    border: 1px solid #a9a496;
    background-image: linear-gradient(#fbfbfa, #e6e3da);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.9), 0 1px 2px rgba(0,0,0,0.2); }
.chip-filled { width: 112px; border: 1px solid #9bb7dd;
    background-image: linear-gradient(#dbe7f8, #b9cdec); }
.chip-filled .chip-text { color: #234a86; }
.chip-check { height: 16px; margin-right: 6px; color: #234a86; font-size: 12px; }
.chip-outlined { width: 104px; }
.chip-outlined .chip-text { color: #4a4636; }
.chip-tonal { background-image: linear-gradient(#dbe7f8, #b9cdec); border: 1px solid #9bb7dd; }
.chip-tonal .chip-text { color: #234a86; }
.chip-wide { width: 172px; }
.chip-text { height: 18px; font-size: 14px; font-weight: 700; }

/* ---- Card & list ---- */
.mini-card { display: flex; flex-direction: column; height: 208px; border-radius: 10px;
    border: 1px solid #a9a496; background-image: linear-gradient(#f6f4ee, #e8e6df);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.7), 0 2px 4px rgba(0,0,0,0.22); }
.mini-cover { display: flex; align-items: center; justify-content: center; height: 84px;
    border-radius: 10px 10px 0 0;
    background-image: linear-gradient(#7eb4f2, #3a78db);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.4); }
.mini-cover-text { height: 16px; color: #ffffff; font-size: 11px; text-align: center; }
.mini-body { display: flex; flex-direction: column; height: 124px; padding: 16px; }
.mini-title { height: 22px; color: #3a362e; font-size: 16px; font-weight: 700; }
.mini-sub { height: 18px; margin-top: 4px; color: #6b6658; font-size: 13px; }
.mini-actions { display: flex; flex-direction: row; align-items: center; height: 38px; margin-top: 14px; }
.people { display: flex; flex-direction: column; height: 100px; margin-top: 14px; }
.person { display: flex; flex-direction: row; align-items: center; height: 50px; }
.person-meta { display: flex; flex-direction: column; justify-content: center; height: 50px;
    margin-left: 13px; width: 420px; }
.person-name { height: 18px; color: #3a362e; font-size: 14px; font-weight: 700; }
.person-email { height: 16px; margin-top: 2px; color: #6b6658; font-size: 12px; }
.person-menu { width: 24px; height: 20px; color: #8a8474; font-size: 18px; text-align: right; }
.avatar { display: flex; align-items: center; justify-content: center;
    width: 36px; height: 36px; border-radius: 18px; border: 1px solid rgba(0,0,0,0.2);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.4), 0 1px 2px rgba(0,0,0,0.3); }
.avatar-text { height: 16px; font-size: 13px; font-weight: 700; text-align: center; color: #ffffff; }
.avatar-ak { background-image: linear-gradient(#6ba8ef, #2f6fd6); }
.avatar-mr { background-image: linear-gradient(#c98a97, #a04a5a); }
.avatar-jp { background-image: linear-gradient(#7ec46a, #4e9a3a); }
.avatar-more { background-image: linear-gradient(#f5cd6b, #e0a82f); }
.avatar-more .avatar-text { color: #6b4a00; }

/* ---- Navigation ---- */
.breadcrumb { display: flex; flex-direction: row; align-items: center; height: 20px; margin-bottom: 18px; }
.crumb { height: 18px; margin-right: 8px; color: #6b6658; font-size: 13px; }
.crumb-sep { height: 18px; margin-right: 8px; color: #a9a496; font-size: 13px; }
.crumb-active { height: 18px; color: #2f6fd6; font-size: 13px; font-weight: 700; }
.nav-tabs { display: flex; flex-direction: row; height: 42px;
    border-bottom: 1px solid #c7c3b6; margin-bottom: 14px; }
.nav-tab { display: flex; flex-direction: column; align-items: center; justify-content: flex-end;
    width: 96px; }
.nav-tab-label { height: 24px; color: #6b6658; font-size: 14px; font-weight: 700; text-align: center; }
.nav-tab-label.active { color: #2f6fd6; }
.nav-tab-underline { width: 72px; height: 3px; border-radius: 3px 3px 0 0;
    background-image: linear-gradient(#7eb4f2, #3a78db); }
.nav-list { display: flex; flex-direction: column; width: 260px; }
.nav-item { display: flex; flex-direction: row; align-items: center; height: 44px;
    border-radius: 10px; padding-left: 16px; padding-right: 16px; margin-bottom: 4px;
    border: 1px solid transparent; }
.nav-item.active { border: 1px solid #a9a496;
    background-image: linear-gradient(#f2efe8, #e0dccf);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.8), 0 1px 2px rgba(0,0,0,0.18); }
.nav-item-text { height: 20px; margin-left: 12px; color: #6b6658; font-size: 14px; }
.nav-item.active .nav-item-text { color: #3a362e; font-weight: 700; }
.dot-on { width: 9px; height: 9px; border-radius: 5px; border: 1px solid #2560bf;
    background-image: linear-gradient(#6ba8ef, #2f6fd6); }
.dot-off { width: 9px; height: 9px; border-radius: 5px; border: 1px solid #a9a496;
    background-image: linear-gradient(#e3e0d8, #c9c5bb); }

/* ---- Badges / chips / avatars: glossy candy pills ---- */
.badge-row { display: flex; flex-direction: row; align-items: center; height: 26px; margin-bottom: 22px; }
.badge { display: flex; align-items: center; justify-content: center; height: 26px;
    border-radius: 13px; margin-right: 8px; border: 1px solid rgba(0,0,0,0.25);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.45), 0 1px 2px rgba(0,0,0,0.25); }
.badge-text { height: 16px; font-size: 12px; font-weight: 700; text-align: center; color: #ffffff; }
.badge-success { width: 76px; background-image: linear-gradient(#7ec46a, #4e9a3a); }
.badge-warn { width: 84px; background-image: linear-gradient(#f5cd6b, #e0a82f); }
.badge-warn .badge-text { color: #5a3d00; }
.badge-error { width: 66px; background-image: linear-gradient(#e8896f, #c0392b); }
.badge-neutral { width: 64px; background-image: linear-gradient(#e3e0d8, #c9c5bb); }
.badge-neutral .badge-text { color: #4a4636; }
.badge-count { display: flex; align-items: center; justify-content: center;
    width: 24px; height: 20px; border-radius: 10px; border: 1px solid #8f1f14;
    background-image: linear-gradient(#e8896f, #c0392b);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.4); }
.badge-count-text { height: 14px; color: #ffffff; font-size: 11px; font-weight: 700; text-align: center; }
.avatar-stack { display: flex; flex-direction: row; align-items: center; height: 40px; }
.avatar.stack { border: 2px solid #f2efe8; }
.avatar.overlap { margin-left: -10px; }

/* ---- Sliders / progress: recessed grooves with glossy fills / knobs ---- */
.slider-head { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 18px; margin-bottom: 10px; }
.slider-name { height: 16px; color: #6b6658; font-size: 13px; }
.slider-val { height: 16px; color: #2f6fd6; font-size: 13px; font-weight: 700; text-align: right; }
.slider { position: relative; height: 24px; margin-bottom: 24px; }
.slider-track { position: absolute; left: 0; right: 0; top: 9px; height: 6px;
    border-radius: 3px; border: 1px solid #a9a496;
    background-image: linear-gradient(#c9c5bb, #ddd9d0);
    box-shadow: inset 0 1px 2px rgba(0,0,0,0.3); }
.slider-fill { position: absolute; left: 0; top: 9px; width: 55%; height: 6px;
    border-radius: 3px; border: 1px solid #2560bf;
    background-image: linear-gradient(#6ba8ef, #2f6fd6);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.4); }
.slider-knob { position: absolute; left: 53%; top: 1px; width: 22px; height: 22px;
    border-radius: 11px; border: 1px solid #b8b3a4;
    background-image: linear-gradient(#ffffff, #dedacf);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.9), 0 2px 3px rgba(0,0,0,0.4); }
.progress-label { height: 18px; margin-bottom: 8px; color: #6b6658; font-size: 13px; }
.progress { height: 8px; border-radius: 4px; border: 1px solid #a9a496; margin-bottom: 20px;
    background-image: linear-gradient(#c9c5bb, #ddd9d0);
    box-shadow: inset 0 1px 2px rgba(0,0,0,0.3); }
.progress-fill { width: 72%; height: 8px; border-radius: 4px; border: 1px solid #2560bf;
    background-image: linear-gradient(#6ba8ef, #2f6fd6);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.4); }
.spinner-row { display: flex; flex-direction: row; align-items: center; height: 28px; }
.spinner { width: 28px; height: 28px; border-radius: 14px; margin-right: 14px;
    border: 3px solid #c9c5bb; border-top-color: #2f6fd6; }
.spinner-text { height: 18px; color: #6b6658; font-size: 13px; }

/* ---- Data table: glossy header, ruled rows ---- */
.table { border: 1px solid #a9a496; border-radius: 10px; }
.thead { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr;
    padding: 14px 18px; border-radius: 10px 10px 0 0;
    background-image: linear-gradient(#e8e6df, #d6d2c6);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.7); }
.th { height: 16px; color: #4a4636; font-size: 12px; font-weight: 700; }
.th-right { text-align: right; }
.trow { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr; align-items: center;
    padding: 14px 18px; border-top: 1px solid #cfcabb; }
.td { height: 20px; color: #3a362e; font-size: 14px; }
.td-name { font-weight: 700; }
.td-muted { color: #6b6658; }
.td-mono { color: #3a362e; }
.td-right { text-align: right; }

/* ---- Feedback ---- */
.feedback-row { display: flex; flex-direction: row; align-items: center; height: 40px; }
"#;

/// The complete **Dark SaaS** stylesheet (theme index 2) — a deep-slate dashboard:
/// near-black page (`#0c0e14`), near-black raised surfaces (`#14161f`) with subtle
/// `1px #232634` light-alpha borders, a vivid violet accent (`#7c5cff`, the spec's
/// `oklch(0.66 0.19 280)` / its `rgba(124,92,255)` glow converted to sRGB), light
/// text on dark (`#e6e8ee`) with muted greys (`#8b90a0`, `#6b7180`), and a small
/// 10-14px radius. Inputs are recessed wells (`#0e1017`) a shade darker than the
/// cards. Same semantic classes and shared layout numbers as [`MATERIAL_CSS`]; solid
/// / rgba fills only, so it reproduces faithfully on the CPU backend.
///
/// # Approximated (our engine's known gaps)
/// - Mono eyebrow **`letter-spacing` / `text-transform`** dropped (no engine
///   support; labels are pre-uppercased in the view).
/// - The title's **text-gradient** (`#fff → #b9b4ff`) and the accent **glow**
///   `box-shadow`s are GPU-class; text keeps a solid light fill and glows degrade to
///   the CPU approximation per the effect-split policy.
/// - The card-cover **`repeating-linear-gradient` hatch** → a flat dark fill.
/// - Avatar `oklch()` hues are converted to their nearest sRGB hex.
const DARK_SAAS_CSS: &str = r#"
* { box-sizing: border-box; }
#app { display: flex; flex-direction: column; background-color: #0c0e14; }

/* ---- Switcher bar ---- */
.kit-switcher { display: flex; flex-direction: row; align-items: center; height: 60px;
    padding-left: 18px; padding-right: 18px; background-color: #14161f;
    border-bottom: 1px solid #232634; }
.kit-brand { display: flex; flex-direction: row; align-items: center; width: 60px; height: 60px; }
.kit-logo { width: 11px; height: 11px; border-radius: 4px; background-color: #7c5cff; }
.kit-brand-name { width: 40px; height: 16px; margin-left: 8px; color: #e6e8ee;
    font-size: 13px; font-weight: 700; }
.kit-tabs { display: flex; flex-direction: row; }
.kit-tab { display: flex; flex-direction: column; align-items: center; justify-content: center;
    width: 108px; height: 60px; }
.kit-tab-label { height: 16px; color: #6b7180; font-size: 12px; font-weight: 500; text-align: center; }
.kit-tab.active .kit-tab-label { color: #e6e8ee; }
.kit-underline { width: 60px; height: 2px; border-radius: 2px; background-color: #7c5cff; margin-top: 6px; }

/* ---- Gallery frame ---- */
.m3-page { display: flex; flex-direction: column; padding: 34px 40px 60px; }
.m3-head { display: flex; flex-direction: column; height: 108px; margin-bottom: 26px; }
.m3-eyebrow { height: 16px; color: #6b7180; font-size: 11px; font-weight: 500; }
/* Title text-gradient (#fff->#b9b4ff) is GPU-class -> solid light fill. */
.m3-title { height: 48px; color: #e9e7ff; font-size: 38px; font-weight: 700; }
.m3-sub { height: 22px; color: #8b90a0; font-size: 15px; }

.gallery-grid { display: flex; flex-direction: column; }
.row { display: flex; flex-direction: row; align-items: stretch; margin-bottom: 24px; }
.row.r1 { height: 232px; }
.row.r2 { height: 336px; }
.row.r3 { height: 420px; }
.row.r4 { height: 252px; }

/* Cards: raised near-black surfaces with a subtle light-alpha hairline. */
.card { background-color: #14161f; border-radius: 14px; padding: 22px;
    border: 1px solid #232634; }
.card.col { width: 588px; margin-right: 24px; }
.card.full { width: 1200px; margin-bottom: 24px; }
.card.full.table-c { height: 288px; }
.card.full.feedback-c { height: 132px; }
.card-label { height: 16px; margin-bottom: 16px; color: #6b7180; font-size: 11px; font-weight: 500; }

/* ---- Buttons: violet primary, slate secondary, ghost accent-text ---- */
.btn-row { display: flex; flex-direction: row; align-items: center; height: 56px; }
.btn { display: flex; flex-direction: row; align-items: center; justify-content: center;
    height: 40px; border-radius: 9px; padding-left: 20px; padding-right: 20px; margin-right: 10px; }
.btn-label { height: 18px; font-size: 14px; font-weight: 600; text-align: center; }
.btn-filled { width: 110px; background-color: #7c5cff; }
.btn-filled .btn-label { color: #ffffff; }
.btn-tonal { width: 104px; background-color: #1c1f2b; border: 1px solid #2e3242; }
.btn-tonal .btn-label { color: #e6e8ee; }
.btn-outlined { width: 128px; background-color: #14161f; border: 1px solid #2e3242; }
.btn-outlined .btn-label { color: #b9b4ff; }
.btn-text { width: 92px; padding-left: 16px; padding-right: 16px; background-color: #14161f; }
.btn-text .btn-label { color: #b9b4ff; }
.btn-dialog { width: 152px; }
.btn-elevated { width: 132px; background-color: #1c1f2b; border: 1px solid #2e3242; }
.btn-elevated .btn-label { color: #e6e8ee; }
.btn-disabled { width: 128px; background-color: #171922; border: 1px solid #20232e; }
.btn-disabled .btn-label { color: #4a4f60; }
/* FAB -> the primary accent add-button (glow degrades on CPU). */
.fab { display: flex; flex-direction: row; align-items: center; justify-content: center;
    width: 56px; height: 56px; border-radius: 10px; background-color: #7c5cff;
    box-shadow: 0 6px 20px rgba(124,92,255,0.45); }
.fab-glyph { height: 30px; color: #ffffff; font-size: 26px; font-weight: 400; text-align: center; }
.btn-sm { height: 38px; }
.btn-wide { width: 168px; }

/* ---- Selection ---- */
.sel-cols { display: flex; flex-direction: row; }
.sel-col { display: flex; flex-direction: column; width: 168px; }
.sel-item { display: flex; flex-direction: row; align-items: center; height: 34px; }
.sel-col .sel-item { margin-bottom: 14px; }
.sel-col.wide-gap .sel-item { margin-bottom: 18px; height: 32px; }
.sel-text { height: 20px; margin-left: 11px; color: #e6e8ee; font-size: 14px; }
.checkbox-on { display: flex; align-items: center; justify-content: center;
    width: 18px; height: 18px; border-radius: 6px; background-color: #7c5cff; }
.check-glyph { height: 14px; color: #ffffff; font-size: 12px; text-align: center; }
.checkbox-off { width: 18px; height: 18px; border-radius: 6px; border: 1px solid #3a3f52;
    background-color: #1c1f2b; }
.checkbox-disabled { width: 18px; height: 18px; border-radius: 6px; border: 1px solid #20232e;
    background-color: #171922; }
.radio-on { display: flex; align-items: center; justify-content: center;
    width: 20px; height: 20px; border-radius: 10px; border: 1px solid #3a3f52; background-color: #1c1f2b; }
.radio-dot { width: 10px; height: 10px; border-radius: 5px; background-color: #7c5cff; }
.radio-off { width: 20px; height: 20px; border-radius: 10px; border: 1px solid #3a3f52; background-color: #1c1f2b; }
.toggle-on { display: flex; flex-direction: row; align-items: center; justify-content: flex-end;
    width: 44px; height: 24px; border-radius: 12px; background-color: #7c5cff; padding: 3px; }
.toggle-knob { width: 18px; height: 18px; border-radius: 9px; background-color: #ffffff; }
.toggle-off { display: flex; flex-direction: row; align-items: center; justify-content: flex-start;
    width: 44px; height: 24px; border-radius: 12px; background-color: #2e3242; padding: 3px; }
.toggle-knob-off { width: 18px; height: 18px; border-radius: 9px; background-color: #8b90a0; }

/* ---- Text fields: recessed dark wells ---- */
.field-col { display: flex; flex-direction: column; }
.field { margin-bottom: 16px; }
.field-filled { background-color: #0e1017; border-radius: 9px; border: 1px solid #2e3242; padding: 9px 12px; }
.field-outlined { background-color: #0e1017; border: 1px solid #2e3242; border-radius: 9px; padding: 9px 12px; }
.field-error { background-color: #1c1216; border: 1px solid #d24f4f; border-radius: 9px; padding: 9px 12px; }
.field-label-primary { height: 14px; margin-bottom: 4px; color: #8b90a0; font-size: 11px; }
.field-label { height: 14px; margin-bottom: 4px; color: #8b90a0; font-size: 11px; }
.field-label-error { height: 14px; margin-bottom: 4px; color: #ff8b8b; font-size: 11px; }
.field-value { height: 20px; color: #e6e8ee; font-size: 15px; }
.field-placeholder { height: 20px; color: #6b7180; font-size: 15px; }
.field-error-text { height: 14px; margin-top: 5px; color: #ff8b8b; font-size: 11px; }

/* ---- Menu / dropdown ---- */
.select-box { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    width: 260px; height: 44px; background-color: #0e1017; border: 1px solid #2e3242; border-radius: 9px; padding: 0 14px; }
.select-value { height: 20px; color: #e6e8ee; font-size: 15px; }
.select-chevron { width: 16px; height: 20px; color: #6b7180; font-size: 14px; text-align: right; }
.menu-panel { display: flex; flex-direction: column; width: 260px; height: 136px; margin-top: 8px;
    background-color: #1a1d28; border: 1px solid #2e3242; border-radius: 9px; padding-top: 6px; padding-bottom: 6px;
    box-shadow: 0 12px 30px rgba(0,0,0,0.5); }
.menu-item { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 40px; padding: 0 12px; }
.menu-item.selected { background-color: #242838; border-radius: 6px; }
.menu-text { height: 20px; color: #e6e8ee; font-size: 15px; }
.menu-check { width: 16px; height: 20px; color: #b9b4ff; font-size: 14px; text-align: right; }
.chips-row { display: flex; flex-direction: row; align-items: center; height: 32px; margin-top: 16px; }
.chip { display: flex; flex-direction: row; align-items: center; justify-content: center; height: 32px;
    border-radius: 8px; padding-left: 12px; padding-right: 12px; margin-right: 8px; }
.chip-filled { width: 112px; background-color: #1c1f2b; border: 1px solid #2e3242; }
.chip-filled .chip-text { color: #e6e8ee; }
.chip-check { height: 16px; margin-right: 6px; color: #6b7180; font-size: 12px; }
.chip-outlined { width: 104px; background-color: #14161f; border: 1px solid #2e3242; }
.chip-outlined .chip-text { color: #e6e8ee; }
.chip-tonal { background-color: rgba(124,92,255,0.14); border: 1px solid #2e3242; }
.chip-tonal .chip-text { color: #cfc9ff; }
.chip-wide { width: 172px; }
.chip-text { height: 18px; font-size: 14px; font-weight: 500; }

/* ---- Card & list ---- */
.mini-card { display: flex; flex-direction: column; height: 208px; background-color: #0e1017;
    border: 1px solid #232634; border-radius: 12px; }
/* cover hatch -> flat dark fill (repeating-linear-gradient unsupported). */
.mini-cover { display: flex; align-items: center; justify-content: center; height: 84px;
    border-radius: 12px 12px 0 0; background-color: #1a1d28; }
.mini-cover-text { height: 16px; color: #4a4f60; font-size: 11px; text-align: center; }
.mini-body { display: flex; flex-direction: column; height: 124px; padding: 16px; }
.mini-title { height: 22px; color: #e6e8ee; font-size: 16px; font-weight: 600; }
.mini-sub { height: 18px; margin-top: 4px; color: #8b90a0; font-size: 13px; }
.mini-actions { display: flex; flex-direction: row; align-items: center; height: 38px; margin-top: 14px; }
.people { display: flex; flex-direction: column; height: 100px; margin-top: 14px; }
.person { display: flex; flex-direction: row; align-items: center; height: 50px; }
.person-meta { display: flex; flex-direction: column; justify-content: center; height: 50px;
    margin-left: 13px; width: 420px; }
.person-name { height: 18px; color: #e6e8ee; font-size: 14px; }
.person-email { height: 16px; margin-top: 2px; color: #8b90a0; font-size: 12px; }
.person-menu { width: 24px; height: 20px; color: #6b7180; font-size: 18px; text-align: right; }
.avatar { display: flex; align-items: center; justify-content: center;
    width: 36px; height: 36px; border-radius: 18px; }
.avatar-text { height: 16px; font-size: 13px; font-weight: 600; text-align: center; color: #ffffff; }
.avatar-ak { background-color: #c26a52; }
.avatar-mr { background-color: #2fa06e; }
.avatar-jp { background-color: #7b6fce; }
.avatar-more { background-color: #1c1f2b; }
.avatar-more .avatar-text { color: #8b90a0; }

/* ---- Navigation ---- */
.breadcrumb { display: flex; flex-direction: row; align-items: center; height: 20px; margin-bottom: 18px; }
.crumb { height: 18px; margin-right: 8px; color: #8b90a0; font-size: 13px; }
.crumb-sep { height: 18px; margin-right: 8px; color: #3a3f52; font-size: 13px; }
.crumb-active { height: 18px; color: #e6e8ee; font-size: 13px; font-weight: 500; }
.nav-tabs { display: flex; flex-direction: row; height: 42px;
    border-bottom: 1px solid #232634; margin-bottom: 14px; }
.nav-tab { display: flex; flex-direction: column; align-items: center; justify-content: flex-end;
    width: 96px; }
.nav-tab-label { height: 24px; color: #6b7180; font-size: 14px; font-weight: 600; text-align: center; }
.nav-tab-label.active { color: #e6e8ee; }
.nav-tab-underline { width: 80px; height: 2px; border-radius: 2px; background-color: #7c5cff; }
.nav-list { display: flex; flex-direction: column; width: 260px; }
.nav-item { display: flex; flex-direction: row; align-items: center; height: 40px;
    border-radius: 8px; padding-left: 12px; padding-right: 12px; margin-bottom: 4px; }
.nav-item.active { background-color: rgba(124,92,255,0.14); }
.nav-item-text { height: 20px; margin-left: 10px; color: #8b90a0; font-size: 14px; }
.nav-item.active .nav-item-text { color: #cfc9ff; font-weight: 600; }
.dot-on { width: 6px; height: 6px; border-radius: 3px; background-color: #7c5cff; }
.dot-off { width: 6px; height: 6px; border-radius: 3px; background-color: #3a3f52; }

/* ---- Badges / chips / avatars: translucent status pills ---- */
.badge-row { display: flex; flex-direction: row; align-items: center; height: 26px; margin-bottom: 22px; }
.badge { display: flex; align-items: center; justify-content: center; height: 24px;
    border-radius: 20px; margin-right: 8px; }
.badge-text { height: 16px; font-size: 12px; font-weight: 600; text-align: center; }
.badge-success { width: 76px; background-color: rgba(52,211,153,0.14); border: 1px solid rgba(52,211,153,0.25); }
.badge-success .badge-text { color: #6ee7b7; }
.badge-warn { width: 84px; background-color: rgba(251,191,36,0.14); border: 1px solid rgba(251,191,36,0.25); }
.badge-warn .badge-text { color: #fcd34d; }
.badge-error { width: 66px; background-color: rgba(248,113,113,0.14); border: 1px solid rgba(248,113,113,0.25); }
.badge-error .badge-text { color: #fca5a5; }
.badge-neutral { width: 64px; background-color: #1c1f2b; border: 1px solid #2e3242; }
.badge-neutral .badge-text { color: #8b90a0; }
.badge-count { display: flex; align-items: center; justify-content: center;
    width: 24px; height: 20px; border-radius: 6px; background-color: #7c5cff; }
.badge-count-text { height: 14px; color: #ffffff; font-size: 11px; font-weight: 700; text-align: center; }
.avatar-stack { display: flex; flex-direction: row; align-items: center; height: 40px; }
.avatar.stack { border: 2px solid #14161f; }
.avatar.overlap { margin-left: -9px; }

/* ---- Sliders / progress ---- */
.slider-head { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 18px; margin-bottom: 10px; }
.slider-name { height: 16px; color: #8b90a0; font-size: 13px; }
.slider-val { height: 16px; color: #cfc9ff; font-size: 13px; text-align: right; }
.slider { position: relative; height: 24px; margin-bottom: 24px; }
.slider-track { position: absolute; left: 0; right: 0; top: 9px; height: 5px;
    border-radius: 3px; background-color: #2e3242; }
.slider-fill { position: absolute; left: 0; top: 9px; width: 55%; height: 5px;
    border-radius: 3px; background-color: #7c5cff; }
.slider-knob { position: absolute; left: 53%; top: 3px; width: 17px; height: 17px;
    border-radius: 9px; background-color: #ffffff; box-shadow: 0 0 0 4px rgba(124,92,255,0.25); }
.progress-label { height: 18px; margin-bottom: 8px; color: #8b90a0; font-size: 13px; }
.progress { height: 8px; border-radius: 4px; background-color: #2e3242; margin-bottom: 20px; }
.progress-fill { width: 72%; height: 8px; border-radius: 4px; background-color: #7c5cff; }
.spinner-row { display: flex; flex-direction: row; align-items: center; height: 28px; }
.spinner { width: 24px; height: 24px; border-radius: 12px; margin-right: 14px;
    border: 3px solid #2e3242; border-top-color: #7c5cff; }
.spinner-text { height: 18px; color: #8b90a0; font-size: 13px; }

/* ---- Data table ---- */
.table { border: 1px solid #232634; border-radius: 10px; }
.thead { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr;
    background-color: #0e1017; padding: 12px 16px; border-radius: 10px 10px 0 0; }
.th { height: 16px; color: #6b7180; font-size: 12px; font-weight: 600; }
.th-right { text-align: right; }
.trow { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr; align-items: center;
    padding: 12px 16px; border-top: 1px solid #1c1f2b; }
.td { height: 20px; color: #e6e8ee; font-size: 14px; }
.td-name { font-weight: 500; }
.td-muted { color: #8b90a0; }
.td-mono { color: #cfc9ff; }
.td-right { text-align: right; }

/* ---- Feedback ---- */
.feedback-row { display: flex; flex-direction: row; align-items: center; height: 40px; }
"#;

/// The complete **Lara / PrimeNG** stylesheet (theme index 6) — the PrimeNG "Lara"
/// look: clean light page (`#f8f9fa`), white cards with a `1px #e9ecef` border, a
/// gentle shadow and a small 6px radius, a confident blue primary (`#3B82F6`), calm
/// greys (`#495057` text, `#6c757d`/`#9aa3ad` muted), soft `#ced4da`/`#dee2e6`
/// borders and filled inputs. Status badges are bold *solid* color blocks with white
/// text (Lara's tag style). Same semantic classes and shared layout numbers as
/// [`MATERIAL_CSS`]; solid fills + soft shadows only, faithful on the CPU backend.
///
/// # Approximated (our engine's known gaps)
/// - Mono eyebrow **`letter-spacing` / `text-transform`** dropped (no engine
///   support; labels are pre-uppercased in the view).
/// - Decisive **focus rings** (`0 0 0 3px #bfdbfe`) are interaction-state only and
///   not shown on a still frame.
/// - The card-cover **`repeating-linear-gradient` hatch** -> a flat grey fill.
const LARA_CSS: &str = r#"
* { box-sizing: border-box; }
#app { display: flex; flex-direction: column; background-color: #f8f9fa; }

/* ---- Switcher bar ---- */
.kit-switcher { display: flex; flex-direction: row; align-items: center; height: 60px;
    padding-left: 18px; padding-right: 18px; background-color: #ffffff;
    border-bottom: 1px solid #e9ecef; }
.kit-brand { display: flex; flex-direction: row; align-items: center; width: 60px; height: 60px; }
.kit-logo { width: 11px; height: 11px; border-radius: 4px; background-color: #3B82F6; }
.kit-brand-name { width: 40px; height: 16px; margin-left: 8px; color: #343a40;
    font-size: 13px; font-weight: 700; }
.kit-tabs { display: flex; flex-direction: row; }
.kit-tab { display: flex; flex-direction: column; align-items: center; justify-content: center;
    width: 108px; height: 60px; }
.kit-tab-label { height: 16px; color: #9aa3ad; font-size: 12px; font-weight: 600; text-align: center; }
.kit-tab.active .kit-tab-label { color: #343a40; }
.kit-underline { width: 60px; height: 2px; border-radius: 2px; background-color: #3B82F6; margin-top: 6px; }

/* ---- Gallery frame ---- */
.m3-page { display: flex; flex-direction: column; padding: 34px 40px 60px; }
.m3-head { display: flex; flex-direction: column; height: 108px; margin-bottom: 26px; }
.m3-eyebrow { height: 16px; color: #9aa3ad; font-size: 11px; font-weight: 600; }
.m3-title { height: 48px; color: #343a40; font-size: 38px; font-weight: 700; }
.m3-sub { height: 22px; color: #6c757d; font-size: 15px; }

.gallery-grid { display: flex; flex-direction: column; }
.row { display: flex; flex-direction: row; align-items: stretch; margin-bottom: 24px; }
.row.r1 { height: 232px; }
.row.r2 { height: 336px; }
.row.r3 { height: 420px; }
.row.r4 { height: 252px; }

/* Cards: white, 1px grey border, 6px radius, a gentle shadow. */
.card { background-color: #ffffff; border-radius: 6px; padding: 22px;
    border: 1px solid #e9ecef; box-shadow: 0 1px 2px rgba(0,0,0,0.04); }
.card.col { width: 588px; margin-right: 24px; }
.card.full { width: 1200px; margin-bottom: 24px; }
.card.full.table-c { height: 288px; }
.card.full.feedback-c { height: 132px; }
.card-label { height: 16px; margin-bottom: 16px; color: #9aa3ad; font-size: 11px; font-weight: 600; }

/* ---- Buttons: solid blue primary, grey secondary, outlined + text ---- */
.btn-row { display: flex; flex-direction: row; align-items: center; height: 56px; }
.btn { display: flex; flex-direction: row; align-items: center; justify-content: center;
    height: 40px; border-radius: 6px; padding-left: 20px; padding-right: 20px; margin-right: 10px; }
.btn-label { height: 18px; font-size: 14px; font-weight: 600; text-align: center; }
.btn-filled { width: 110px; background-color: #3B82F6; }
.btn-filled .btn-label { color: #ffffff; }
.btn-tonal { width: 104px; background-color: #64748B; }
.btn-tonal .btn-label { color: #ffffff; }
.btn-outlined { width: 128px; background-color: #ffffff; border: 1px solid #3B82F6; }
.btn-outlined .btn-label { color: #3B82F6; }
.btn-text { width: 92px; padding-left: 16px; padding-right: 16px; background-color: #ffffff; }
.btn-text .btn-label { color: #3B82F6; }
.btn-dialog { width: 152px; }
.btn-elevated { width: 132px; background-color: #22C55E; }
.btn-elevated .btn-label { color: #ffffff; }
.btn-disabled { width: 128px; background-color: #a5c9fb; }
.btn-disabled .btn-label { color: #ffffff; }
/* FAB -> the icon-square add affordance in the Lara set. */
.fab { display: flex; flex-direction: row; align-items: center; justify-content: center;
    width: 56px; height: 56px; border-radius: 6px; background-color: #ffffff; border: 1px solid #ced4da; }
.fab-glyph { height: 30px; color: #495057; font-size: 26px; font-weight: 400; text-align: center; }
.btn-sm { height: 38px; }
.btn-wide { width: 168px; }

/* ---- Selection ---- */
.sel-cols { display: flex; flex-direction: row; }
.sel-col { display: flex; flex-direction: column; width: 168px; }
.sel-item { display: flex; flex-direction: row; align-items: center; height: 34px; }
.sel-col .sel-item { margin-bottom: 14px; }
.sel-col.wide-gap .sel-item { margin-bottom: 18px; height: 32px; }
.sel-text { height: 20px; margin-left: 11px; color: #495057; font-size: 14px; }
.checkbox-on { display: flex; align-items: center; justify-content: center;
    width: 20px; height: 20px; border-radius: 6px; background-color: #3B82F6; }
.check-glyph { height: 14px; color: #ffffff; font-size: 12px; text-align: center; }
.checkbox-off { width: 20px; height: 20px; border-radius: 6px; border: 2px solid #ced4da; background-color: #ffffff; }
.checkbox-disabled { width: 20px; height: 20px; border-radius: 6px; border: 2px solid #e9ecef; background-color: #e9ecef; }
.radio-on { display: flex; align-items: center; justify-content: center;
    width: 20px; height: 20px; border-radius: 10px; border: 2px solid #ced4da; background-color: #ffffff; }
.radio-dot { width: 10px; height: 10px; border-radius: 5px; background-color: #3B82F6; }
.radio-off { width: 20px; height: 20px; border-radius: 10px; border: 2px solid #ced4da; background-color: #ffffff; }
.toggle-on { display: flex; flex-direction: row; align-items: center; justify-content: flex-end;
    width: 46px; height: 26px; border-radius: 13px; background-color: #3B82F6; padding: 3px; }
.toggle-knob { width: 20px; height: 20px; border-radius: 10px; background-color: #ffffff; }
.toggle-off { display: flex; flex-direction: row; align-items: center; justify-content: flex-start;
    width: 46px; height: 26px; border-radius: 13px; background-color: #cbd5e1; padding: 3px; }
.toggle-knob-off { width: 20px; height: 20px; border-radius: 10px; background-color: #ffffff; }

/* ---- Text fields: filled white inputs, 6px radius, grey borders ---- */
.field-col { display: flex; flex-direction: column; }
.field { margin-bottom: 16px; }
.field-filled { background-color: #ffffff; border-radius: 6px; border: 1px solid #ced4da; padding: 9px 12px; }
.field-outlined { background-color: #ffffff; border: 1px solid #ced4da; border-radius: 6px; padding: 9px 12px; }
.field-error { background-color: #ffffff; border: 1px solid #EF4444; border-radius: 6px; padding: 9px 12px; }
.field-label-primary { height: 14px; margin-bottom: 4px; color: #495057; font-size: 11px; font-weight: 600; }
.field-label { height: 14px; margin-bottom: 4px; color: #495057; font-size: 11px; font-weight: 600; }
.field-label-error { height: 14px; margin-bottom: 4px; color: #EF4444; font-size: 11px; font-weight: 600; }
.field-value { height: 20px; color: #495057; font-size: 15px; }
.field-placeholder { height: 20px; color: #9aa3ad; font-size: 15px; }
.field-error-text { height: 14px; margin-top: 5px; color: #EF4444; font-size: 11px; }

/* ---- Menu / dropdown ---- */
.select-box { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    width: 260px; height: 44px; background-color: #ffffff; border: 1px solid #ced4da; border-radius: 6px; padding: 0 14px; }
.select-value { height: 20px; color: #495057; font-size: 15px; }
.select-chevron { width: 16px; height: 20px; color: #9aa3ad; font-size: 14px; text-align: right; }
.menu-panel { display: flex; flex-direction: column; width: 260px; height: 136px; margin-top: 8px;
    background-color: #ffffff; border: 1px solid #e9ecef; border-radius: 6px; padding-top: 6px; padding-bottom: 6px;
    box-shadow: 0 10px 28px rgba(0,0,0,0.12); }
.menu-item { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 40px; padding: 0 12px; }
.menu-item.selected { background-color: #eff6ff; border-radius: 6px; }
.menu-text { height: 20px; color: #495057; font-size: 15px; }
.menu-check { width: 16px; height: 20px; color: #3B82F6; font-size: 14px; text-align: right; }
.chips-row { display: flex; flex-direction: row; align-items: center; height: 32px; margin-top: 16px; }
.chip { display: flex; flex-direction: row; align-items: center; justify-content: center; height: 32px;
    border-radius: 16px; padding-left: 14px; padding-right: 14px; margin-right: 8px; }
.chip-filled { width: 112px; background-color: #dee2e6; }
.chip-filled .chip-text { color: #495057; }
.chip-check { height: 16px; margin-right: 6px; color: #6c757d; font-size: 12px; }
.chip-outlined { width: 104px; background-color: #ffffff; border: 1px solid #dee2e6; }
.chip-outlined .chip-text { color: #495057; }
.chip-tonal { background-color: #dee2e6; }
.chip-tonal .chip-text { color: #495057; }
.chip-wide { width: 172px; }
.chip-text { height: 18px; font-size: 14px; font-weight: 600; }

/* ---- Card & list ---- */
.mini-card { display: flex; flex-direction: column; height: 208px; background-color: #ffffff;
    border: 1px solid #e9ecef; border-radius: 6px; box-shadow: 0 1px 3px rgba(0,0,0,0.06); }
/* cover hatch -> flat grey fill (repeating-linear-gradient unsupported). */
.mini-cover { display: flex; align-items: center; justify-content: center; height: 84px;
    border-radius: 6px 6px 0 0; background-color: #eef1f4; }
.mini-cover-text { height: 16px; color: #adb5bd; font-size: 11px; text-align: center; }
.mini-body { display: flex; flex-direction: column; height: 124px; padding: 16px; }
.mini-title { height: 22px; color: #343a40; font-size: 16px; font-weight: 700; }
.mini-sub { height: 18px; margin-top: 4px; color: #6c757d; font-size: 13px; }
.mini-actions { display: flex; flex-direction: row; align-items: center; height: 38px; margin-top: 14px; }
.people { display: flex; flex-direction: column; height: 100px; margin-top: 14px; }
.person { display: flex; flex-direction: row; align-items: center; height: 50px; }
.person-meta { display: flex; flex-direction: column; justify-content: center; height: 50px;
    margin-left: 13px; width: 420px; }
.person-name { height: 18px; color: #495057; font-size: 14px; }
.person-email { height: 16px; margin-top: 2px; color: #9aa3ad; font-size: 12px; }
.person-menu { width: 24px; height: 20px; color: #9aa3ad; font-size: 18px; text-align: right; }
.avatar { display: flex; align-items: center; justify-content: center;
    width: 36px; height: 36px; border-radius: 18px; }
.avatar-text { height: 16px; font-size: 13px; font-weight: 600; text-align: center; color: #ffffff; }
.avatar-ak { background-color: #3B82F6; }
.avatar-mr { background-color: #22C55E; }
.avatar-jp { background-color: #8b5cf6; }
.avatar-more { background-color: #e9ecef; }
.avatar-more .avatar-text { color: #6c757d; }

/* ---- Navigation ---- */
.breadcrumb { display: flex; flex-direction: row; align-items: center; height: 20px; margin-bottom: 18px; }
.crumb { height: 18px; margin-right: 8px; color: #3B82F6; font-size: 13px; }
.crumb-sep { height: 18px; margin-right: 8px; color: #ced4da; font-size: 13px; }
.crumb-active { height: 18px; color: #495057; font-size: 13px; font-weight: 600; }
.nav-tabs { display: flex; flex-direction: row; height: 42px;
    border-bottom: 2px solid #e9ecef; margin-bottom: 14px; }
.nav-tab { display: flex; flex-direction: column; align-items: center; justify-content: flex-end;
    width: 96px; }
.nav-tab-label { height: 24px; color: #6c757d; font-size: 14px; font-weight: 600; text-align: center; }
.nav-tab-label.active { color: #3B82F6; }
.nav-tab-underline { width: 96px; height: 2px; background-color: #3B82F6; }
.nav-list { display: flex; flex-direction: column; width: 260px; }
.nav-item { display: flex; flex-direction: row; align-items: center; height: 40px;
    border-radius: 6px; padding-left: 12px; padding-right: 12px; margin-bottom: 4px; }
.nav-item.active { background-color: #eff6ff; }
.nav-item-text { height: 20px; margin-left: 10px; color: #495057; font-size: 14px; }
.nav-item.active .nav-item-text { color: #1d4ed8; font-weight: 600; }
.dot-on { width: 6px; height: 6px; border-radius: 3px; background-color: #3B82F6; }
.dot-off { width: 6px; height: 6px; border-radius: 3px; background-color: #ced4da; }

/* ---- Badges / chips / avatars: bold solid tags with white text, 6px radius ---- */
.badge-row { display: flex; flex-direction: row; align-items: center; height: 26px; margin-bottom: 22px; }
.badge { display: flex; align-items: center; justify-content: center; height: 24px;
    border-radius: 6px; margin-right: 8px; }
.badge-text { height: 16px; font-size: 12px; font-weight: 700; text-align: center; }
.badge-success { width: 76px; background-color: #22C55E; }
.badge-success .badge-text { color: #ffffff; }
.badge-warn { width: 84px; background-color: #F59E0B; }
.badge-warn .badge-text { color: #ffffff; }
.badge-error { width: 66px; background-color: #EF4444; }
.badge-error .badge-text { color: #ffffff; }
.badge-neutral { width: 64px; background-color: #e9ecef; }
.badge-neutral .badge-text { color: #6c757d; }
.badge-count { display: flex; align-items: center; justify-content: center;
    width: 24px; height: 20px; border-radius: 10px; background-color: #EF4444; }
.badge-count-text { height: 14px; color: #ffffff; font-size: 11px; font-weight: 700; text-align: center; }
.avatar-stack { display: flex; flex-direction: row; align-items: center; height: 40px; }
.avatar.stack { border: 2px solid #ffffff; }
.avatar.overlap { margin-left: -10px; }

/* ---- Sliders / progress ---- */
.slider-head { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 18px; margin-bottom: 10px; }
.slider-name { height: 16px; color: #6c757d; font-size: 13px; }
.slider-val { height: 16px; color: #3B82F6; font-size: 13px; font-weight: 600; text-align: right; }
.slider { position: relative; height: 24px; margin-bottom: 24px; }
.slider-track { position: absolute; left: 0; right: 0; top: 9px; height: 6px;
    border-radius: 3px; background-color: #dee2e6; }
.slider-fill { position: absolute; left: 0; top: 9px; width: 55%; height: 6px;
    border-radius: 3px; background-color: #3B82F6; }
.slider-knob { position: absolute; left: 53%; top: 3px; width: 18px; height: 18px;
    border-radius: 9px; background-color: #3B82F6; box-shadow: 0 0 0 4px rgba(59,130,246,0.20); }
.progress-label { height: 18px; margin-bottom: 8px; color: #6c757d; font-size: 13px; }
.progress { height: 8px; border-radius: 4px; background-color: #dee2e6; margin-bottom: 20px; }
.progress-fill { width: 72%; height: 8px; border-radius: 4px; background-color: #3B82F6; }
.spinner-row { display: flex; flex-direction: row; align-items: center; height: 28px; }
.spinner { width: 24px; height: 24px; border-radius: 12px; margin-right: 14px;
    border: 3px solid #dee2e6; border-top-color: #3B82F6; }
.spinner-text { height: 18px; color: #6c757d; font-size: 13px; }

/* ---- Data table ---- */
.table { border: 1px solid #e9ecef; border-radius: 6px; }
.thead { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr;
    background-color: #f8f9fa; padding: 13px 16px; border-radius: 6px 6px 0 0; }
.th { height: 16px; color: #343a40; font-size: 12px; font-weight: 700; }
.th-right { text-align: right; }
.trow { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr; align-items: center;
    padding: 13px 16px; border-top: 1px solid #f1f3f5; }
.td { height: 20px; color: #495057; font-size: 14px; }
.td-name { color: #343a40; font-weight: 600; }
.td-muted { color: #6c757d; }
.td-mono { color: #495057; }
.td-right { text-align: right; }

/* ---- Feedback ---- */
.feedback-row { display: flex; flex-direction: row; align-items: center; height: 40px; }
"#;

/// The complete **Fluent / Acrylic** stylesheet (theme index 7) — the eighth and
/// final UI-Kit theme, and the one that exercises the freshly-shipped
/// `backdrop-filter: blur()` (frosted-glass / acrylic material). Windows-11
/// Fluent: Segoe UI face (approximated by the host font), a single vivid blue
/// accent (`#0F6CBD`), a red error (`#C50F1F`), soft 6–12px rounding, thin light
/// 1px borders, and translucent surfaces layered over a light "mica" page wash.
///
/// # Acrylic — how the frost is made to READ
///
/// `backdrop-filter: blur()` only shows if there is *variation* behind the panel
/// (blurring a flat fill returns the same fill). So the page (`#app`) is painted
/// with a **varied light gradient** — a gentle diagonal blue→neutral→violet→teal
/// mica wash over `#eef1f6` — and the cards/menus/dialog are **acrylic**:
/// a translucent `rgba(255,255,255,0.55)` fill + `backdrop-filter: blur(24px)` +
/// a hairline light border. The gradient shows through each card *frosted and
/// tinted*, which is the acrylic. This is GPU-only: the Vello backend frosts the
/// backdrop; the CPU backend honestly skips the blur (the fill is still
/// translucent, so the tint still bleeds — just un-blurred).
///
/// Layout numbers (row heights, card widths, control sizes) are shared with the
/// other themes so the fixed [`view`] fits. Fluent-specific approximations:
/// - **Segoe UI / JetBrains Mono** → the host CPU font (shapes, not the face).
/// - **`letter-spacing` / `text-transform`** (eyebrow tracking, uppercasing) are
///   not in our property set → dropped; eyebrow labels are already uppercased in
///   the view. **`backdrop-filter: saturate()`** (spec pairs it with blur) is not
///   modelled → silently dropped; only the `blur()` is honored.
/// - The spec's absolutely-positioned blurred colour *blobs* can't be added to the
///   fixed view → their vivid tint is folded into the `#app` mica gradient instead.
const FLUENT_CSS: &str = r#"
* { box-sizing: border-box; }
/* Page = a light "mica" wash: a varied diagonal gradient (blue→neutral→violet→
   neutral→teal) over #eef1f6 so the acrylic frost has something to blur/tint. */
#app { display: flex; flex-direction: column; background-color: #eef1f6;
    background-image: linear-gradient(125deg, #cfe0f7 0%, #eef1f6 27%, #e7d8f2 52%, #eef1f6 74%, #d2ece8 100%); }

/* ---- Switcher bar: acrylic (frosts the mica wash under the title bar) ---- */
.kit-switcher { display: flex; flex-direction: row; align-items: center; height: 60px;
    padding-left: 18px; padding-right: 18px;
    background-color: rgba(255,255,255,0.60); backdrop-filter: blur(24px);
    border-bottom: 1px solid rgba(255,255,255,0.65); }
.kit-brand { display: flex; flex-direction: row; align-items: center; width: 60px; height: 60px; }
.kit-logo { width: 11px; height: 11px; border-radius: 3px; background-color: #0F6CBD; }
.kit-brand-name { width: 40px; height: 16px; margin-left: 8px; color: #242424;
    font-size: 13px; font-weight: 700; }
.kit-tabs { display: flex; flex-direction: row; }
.kit-tab { display: flex; flex-direction: column; align-items: center; justify-content: center;
    width: 108px; height: 60px; }
.kit-tab-label { height: 16px; color: #5a6270; font-size: 12px; font-weight: 600; text-align: center; }
.kit-tab.active .kit-tab-label { color: #0F6CBD; }
.kit-underline { width: 60px; height: 2px; border-radius: 2px; background-color: #0F6CBD; margin-top: 6px; }

/* ---- Gallery frame (transparent, so the mica wash shows behind the cards) ---- */
.m3-page { display: flex; flex-direction: column; padding: 34px 40px 60px; }
.m3-head { display: flex; flex-direction: column; height: 108px; margin-bottom: 26px; }
.m3-eyebrow { height: 16px; color: #5a6270; font-size: 11px; font-weight: 600; }
.m3-title { height: 48px; color: #242424; font-size: 38px; font-weight: 600; }
.m3-sub { height: 22px; color: #4b5563; font-size: 15px; }

.gallery-grid { display: flex; flex-direction: column; }
.row { display: flex; flex-direction: row; align-items: stretch; margin-bottom: 24px; }
.row.r1 { height: 232px; }
.row.r2 { height: 336px; }
.row.r3 { height: 420px; }
.row.r4 { height: 252px; }

/* Cards = ACRYLIC: translucent white + backdrop blur + hairline light border +
   soft depth shadow. The mica gradient frosts + tints through each panel. */
.card { background-color: rgba(255,255,255,0.55); backdrop-filter: blur(24px);
    border: 1px solid rgba(255,255,255,0.65); border-radius: 12px; padding: 22px;
    box-shadow: 0 8px 24px rgba(15,30,60,0.10), 0 1px 2px rgba(0,0,0,0.06); }
.card.col { width: 588px; margin-right: 24px; }
.card.full { width: 1200px; margin-bottom: 24px; }
.card.full.table-c { height: 288px; }
.card.full.feedback-c { height: 132px; }
.card-label { height: 16px; margin-bottom: 16px; color: #6b7280; font-size: 11px; font-weight: 600; }

/* ---- Buttons: accent-filled primary, translucent standard, subtle text ---- */
.btn-row { display: flex; flex-direction: row; align-items: center; height: 56px; }
.btn { display: flex; flex-direction: row; align-items: center; justify-content: center;
    height: 40px; border-radius: 6px; padding-left: 20px; padding-right: 20px; margin-right: 10px; }
.btn-label { height: 18px; font-size: 14px; font-weight: 600; text-align: center; }
.btn-filled { width: 110px; background-color: #0F6CBD; box-shadow: 0 2px 4px rgba(15,108,189,0.32); }
.btn-filled .btn-label { color: #ffffff; }
.btn-tonal { width: 104px; background-color: rgba(255,255,255,0.70); border: 1px solid rgba(0,0,0,0.14); }
.btn-tonal .btn-label { color: #242424; }
.btn-outlined { width: 128px; background-color: rgba(255,255,255,0.55); border: 1px solid #0F6CBD; }
.btn-outlined .btn-label { color: #0F6CBD; }
.btn-text { width: 92px; padding-left: 16px; padding-right: 16px; background-color: rgba(255,255,255,0.0); }
.btn-text .btn-label { color: #0F6CBD; }
.btn-dialog { width: 152px; }
.btn-elevated { width: 132px; background-color: rgba(255,255,255,0.70); border: 1px solid rgba(0,0,0,0.14);
    box-shadow: 0 4px 12px rgba(15,30,60,0.10); }
.btn-elevated .btn-label { color: #0F6CBD; }
.btn-disabled { width: 128px; background-color: rgba(0,0,0,0.03); border: 1px solid rgba(0,0,0,0.08); }
.btn-disabled .btn-label { color: #bdbdbd; }
/* FAB -> the Fluent accent icon-square add affordance. */
.fab { display: flex; flex-direction: row; align-items: center; justify-content: center;
    width: 56px; height: 56px; border-radius: 8px; background-color: #0F6CBD;
    box-shadow: 0 4px 12px rgba(15,108,189,0.34); }
.fab-glyph { height: 30px; color: #ffffff; font-size: 26px; font-weight: 400; text-align: center; }
.btn-sm { height: 38px; }
.btn-wide { width: 168px; }

/* ---- Selection ---- */
.sel-cols { display: flex; flex-direction: row; }
.sel-col { display: flex; flex-direction: column; width: 168px; }
.sel-item { display: flex; flex-direction: row; align-items: center; height: 34px; }
.sel-col .sel-item { margin-bottom: 14px; }
.sel-col.wide-gap .sel-item { margin-bottom: 18px; height: 32px; }
.sel-text { height: 20px; margin-left: 11px; color: #242424; font-size: 14px; }
.checkbox-on { display: flex; align-items: center; justify-content: center;
    width: 19px; height: 19px; border-radius: 4px; background-color: #0F6CBD; }
.check-glyph { height: 14px; color: #ffffff; font-size: 12px; text-align: center; }
.checkbox-off { width: 19px; height: 19px; border-radius: 4px; border: 2px solid #616161; background-color: rgba(255,255,255,0.60); }
.checkbox-disabled { width: 19px; height: 19px; border-radius: 4px; border: 2px solid #d4d4d4; background-color: rgba(0,0,0,0.03); }
.radio-on { display: flex; align-items: center; justify-content: center;
    width: 20px; height: 20px; border-radius: 10px; border: 2px solid #0F6CBD; background-color: rgba(255,255,255,0.60); }
.radio-dot { width: 10px; height: 10px; border-radius: 5px; background-color: #0F6CBD; }
.radio-off { width: 20px; height: 20px; border-radius: 10px; border: 2px solid #616161; background-color: rgba(255,255,255,0.60); }
.toggle-on { display: flex; flex-direction: row; align-items: center; justify-content: flex-end;
    width: 44px; height: 22px; border-radius: 11px; background-color: #0F6CBD; padding: 4px; }
.toggle-knob { width: 14px; height: 14px; border-radius: 7px; background-color: #ffffff; }
.toggle-off { display: flex; flex-direction: row; align-items: center; justify-content: flex-start;
    width: 44px; height: 22px; border-radius: 11px; background-color: rgba(255,255,255,0.50);
    border: 1px solid #616161; padding: 4px; }
.toggle-knob-off { width: 13px; height: 13px; border-radius: 7px; background-color: #616161; }

/* ---- Text fields: translucent inputs, accent underline on the primary/error ---- */
.field-col { display: flex; flex-direction: column; }
.field { margin-bottom: 16px; }
.field-filled { background-color: rgba(255,255,255,0.70); border: 1px solid rgba(0,0,0,0.14);
    border-bottom: 2px solid #0F6CBD; border-radius: 5px; padding: 9px 12px; }
.field-outlined { background-color: rgba(255,255,255,0.70); border: 1px solid rgba(0,0,0,0.14); border-radius: 5px; padding: 9px 12px; }
.field-error { background-color: rgba(255,255,255,0.70); border: 1px solid #C50F1F; border-bottom: 2px solid #C50F1F; border-radius: 5px; padding: 9px 12px; }
.field-label-primary { height: 14px; margin-bottom: 4px; color: #4b5563; font-size: 11px; font-weight: 600; }
.field-label { height: 14px; margin-bottom: 4px; color: #4b5563; font-size: 11px; font-weight: 600; }
.field-label-error { height: 14px; margin-bottom: 4px; color: #C50F1F; font-size: 11px; font-weight: 600; }
.field-value { height: 20px; color: #242424; font-size: 15px; }
.field-placeholder { height: 20px; color: #6b7280; font-size: 15px; }
.field-error-text { height: 14px; margin-top: 5px; color: #C50F1F; font-size: 11px; }

/* ---- Menu / dropdown: acrylic panel over the mica wash ---- */
.select-box { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    width: 260px; height: 44px; background-color: rgba(255,255,255,0.70); border: 1px solid rgba(0,0,0,0.14);
    border-bottom: 2px solid #0F6CBD; border-radius: 5px; padding: 0 14px; }
.select-value { height: 20px; color: #242424; font-size: 15px; }
.select-chevron { width: 16px; height: 20px; color: #6b7280; font-size: 14px; text-align: right; }
.menu-panel { display: flex; flex-direction: column; width: 260px; height: 136px; margin-top: 8px;
    background-color: rgba(250,251,253,0.82); backdrop-filter: blur(24px);
    border: 1px solid rgba(255,255,255,0.70); border-radius: 8px; padding-top: 6px; padding-bottom: 6px;
    box-shadow: 0 12px 32px rgba(15,30,60,0.20); }
.menu-item { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 40px; padding: 0 12px; }
.menu-item.selected { background-color: rgba(15,108,189,0.10); border-radius: 6px; }
.menu-text { height: 20px; color: #242424; font-size: 15px; }
.menu-check { width: 16px; height: 20px; color: #0F6CBD; font-size: 14px; text-align: right; }
.chips-row { display: flex; flex-direction: row; align-items: center; height: 32px; margin-top: 16px; }
.chip { display: flex; flex-direction: row; align-items: center; justify-content: center; height: 30px;
    border-radius: 15px; padding-left: 14px; padding-right: 14px; margin-right: 8px; }
.chip-filled { width: 112px; background-color: rgba(15,108,189,0.14); }
.chip-filled .chip-text { color: #0F6CBD; }
.chip-check { height: 16px; margin-right: 6px; color: #0F6CBD; font-size: 12px; }
.chip-outlined { width: 104px; background-color: rgba(255,255,255,0.70); border: 1px solid rgba(0,0,0,0.14); }
.chip-outlined .chip-text { color: #4b5563; }
.chip-tonal { background-color: rgba(15,108,189,0.14); }
.chip-tonal .chip-text { color: #0F6CBD; }
.chip-wide { width: 172px; }
.chip-text { height: 18px; font-size: 13px; font-weight: 600; }

/* ---- Card & list: a nested translucent tile inside the acrylic card ---- */
.mini-card { display: flex; flex-direction: column; height: 208px; background-color: rgba(255,255,255,0.50);
    border: 1px solid rgba(255,255,255,0.70); border-radius: 10px; box-shadow: 0 4px 12px rgba(15,30,60,0.08); }
.mini-cover { display: flex; align-items: center; justify-content: center; height: 84px;
    border-radius: 10px 10px 0 0; background-image: linear-gradient(120deg, #0F6CBD, #9240C5); }
.mini-cover-text { height: 16px; color: #ffffff; font-size: 11px; text-align: center; }
.mini-body { display: flex; flex-direction: column; height: 124px; padding: 16px; }
.mini-title { height: 22px; color: #242424; font-size: 16px; font-weight: 600; }
.mini-sub { height: 18px; margin-top: 4px; color: #4b5563; font-size: 13px; }
.mini-actions { display: flex; flex-direction: row; align-items: center; height: 38px; margin-top: 14px; }
.people { display: flex; flex-direction: column; height: 100px; margin-top: 14px; }
.person { display: flex; flex-direction: row; align-items: center; height: 50px; }
.person-meta { display: flex; flex-direction: column; justify-content: center; height: 50px;
    margin-left: 13px; width: 420px; }
.person-name { height: 18px; color: #242424; font-size: 14px; }
.person-email { height: 16px; margin-top: 2px; color: #4b5563; font-size: 12px; }
.person-menu { width: 24px; height: 20px; color: #9ca3af; font-size: 18px; text-align: right; }
.avatar { display: flex; align-items: center; justify-content: center;
    width: 36px; height: 36px; border-radius: 18px; }
.avatar-text { height: 16px; font-size: 13px; font-weight: 600; text-align: center; color: #ffffff; }
.avatar-ak { background-color: #0F6CBD; }
.avatar-mr { background-color: #9240C5; }
.avatar-jp { background-color: #107C10; }
.avatar-more { background-color: rgba(15,108,189,0.16); }
.avatar-more .avatar-text { color: #0F6CBD; }

/* ---- Navigation ---- */
.breadcrumb { display: flex; flex-direction: row; align-items: center; height: 20px; margin-bottom: 18px; }
.crumb { height: 18px; margin-right: 8px; color: #4b5563; font-size: 13px; }
.crumb-sep { height: 18px; margin-right: 8px; color: #c4c9d1; font-size: 13px; }
.crumb-active { height: 18px; color: #0F6CBD; font-size: 13px; font-weight: 600; }
.nav-tabs { display: flex; flex-direction: row; height: 42px;
    border-bottom: 1px solid rgba(0,0,0,0.08); margin-bottom: 14px; }
.nav-tab { display: flex; flex-direction: column; align-items: center; justify-content: flex-end;
    width: 96px; }
.nav-tab-label { height: 24px; color: #4b5563; font-size: 14px; font-weight: 600; text-align: center; }
.nav-tab-label.active { color: #0F6CBD; }
.nav-tab-underline { width: 68px; height: 2px; border-radius: 3px; background-color: #0F6CBD; }
.nav-list { display: flex; flex-direction: column; width: 260px; }
.nav-item { display: flex; flex-direction: row; align-items: center; height: 40px;
    border-radius: 6px; padding-left: 14px; padding-right: 14px; margin-bottom: 3px; }
.nav-item.active { background-color: rgba(15,108,189,0.12); }
.nav-item-text { height: 20px; margin-left: 12px; color: #4b5563; font-size: 14px; }
.nav-item.active .nav-item-text { color: #0F6CBD; font-weight: 600; }
.dot-on { width: 8px; height: 8px; border-radius: 2px; background-color: #0F6CBD; }
.dot-off { width: 8px; height: 8px; border-radius: 2px; background-color: #c4c9d1; }

/* ---- Badges / chips / avatars: soft tonal pill tags ---- */
.badge-row { display: flex; flex-direction: row; align-items: center; height: 26px; margin-bottom: 22px; }
.badge { display: flex; align-items: center; justify-content: center; height: 24px;
    border-radius: 12px; margin-right: 8px; }
.badge-text { height: 16px; font-size: 12px; font-weight: 600; text-align: center; }
.badge-success { width: 76px; background-color: rgba(16,124,16,0.14); }
.badge-success .badge-text { color: #0E700E; }
.badge-warn { width: 84px; background-color: rgba(157,93,0,0.15); }
.badge-warn .badge-text { color: #8A5300; }
.badge-error { width: 66px; background-color: rgba(197,15,31,0.13); }
.badge-error .badge-text { color: #C50F1F; }
.badge-neutral { width: 64px; background-color: rgba(0,0,0,0.06); }
.badge-neutral .badge-text { color: #4b5563; }
.badge-count { display: flex; align-items: center; justify-content: center;
    width: 24px; height: 20px; border-radius: 10px; background-color: #C50F1F; }
.badge-count-text { height: 14px; color: #ffffff; font-size: 11px; font-weight: 700; text-align: center; }
.avatar-stack { display: flex; flex-direction: row; align-items: center; height: 40px; }
.avatar.stack { border: 2px solid rgba(255,255,255,0.80); }
.avatar.overlap { margin-left: -10px; }

/* ---- Sliders / progress ---- */
.slider-head { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 18px; margin-bottom: 10px; }
.slider-name { height: 16px; color: #4b5563; font-size: 13px; }
.slider-val { height: 16px; color: #0F6CBD; font-size: 13px; font-weight: 600; text-align: right; }
.slider { position: relative; height: 24px; margin-bottom: 24px; }
.slider-track { position: absolute; left: 0; right: 0; top: 10px; height: 4px;
    border-radius: 2px; background-color: rgba(0,0,0,0.12); }
.slider-fill { position: absolute; left: 0; top: 10px; width: 55%; height: 4px;
    border-radius: 2px; background-color: #0F6CBD; }
.slider-knob { position: absolute; left: 53%; top: 2px; width: 20px; height: 20px;
    border-radius: 10px; background-color: #ffffff; border: 5px solid #0F6CBD; box-shadow: 0 1px 3px rgba(0,0,0,0.20); }
.progress-label { height: 18px; margin-bottom: 8px; color: #4b5563; font-size: 13px; }
.progress { height: 4px; border-radius: 2px; background-color: rgba(0,0,0,0.12); margin-bottom: 20px; }
.progress-fill { width: 72%; height: 4px; border-radius: 2px; background-color: #0F6CBD; }
.spinner-row { display: flex; flex-direction: row; align-items: center; height: 28px; }
.spinner { width: 28px; height: 28px; border-radius: 14px; margin-right: 14px;
    border: 3px solid rgba(0,0,0,0.10); border-top-color: #0F6CBD; }
.spinner-text { height: 18px; color: #4b5563; font-size: 13px; }

/* ---- Data table ---- */
.table { border: 1px solid rgba(0,0,0,0.08); border-radius: 10px; }
.thead { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr;
    background-color: rgba(255,255,255,0.50); padding: 13px 18px; border-radius: 10px 10px 0 0; }
.th { height: 16px; color: #4b5563; font-size: 12px; font-weight: 600; }
.th-right { text-align: right; }
.trow { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr; align-items: center;
    padding: 13px 18px; border-top: 1px solid rgba(0,0,0,0.06); }
.td { height: 20px; color: #242424; font-size: 14px; }
.td-name { color: #242424; font-weight: 600; }
.td-muted { color: #4b5563; }
.td-mono { color: #242424; }
.td-right { text-align: right; }

/* ---- Feedback ---- */
.feedback-row { display: flex; flex-direction: row; align-items: center; height: 40px; }
"#;
