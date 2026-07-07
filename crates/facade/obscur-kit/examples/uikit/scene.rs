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
//! and author its rules against the same class vocabulary. Phase 1 shipped the
//! **Material** entry complete; the **Skeuomorphic** entry (index 4) is now also
//! authored in full (gradients + inset/outset shadows + bevels); the remaining
//! six are neutral placeholders.
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

/// The `(theme_name, stylesheet)` table. Index [`MATERIAL_INDEX`] is the complete
/// Material 3 stylesheet and index 4 the complete **Skeuomorphic** stylesheet
/// ([`SKEUOMORPHIC_CSS`]); every other slot is a neutral placeholder sharing the
/// same class vocabulary. Adding a real theme later = swapping one entry's CSS.
#[must_use]
pub fn themes() -> &'static [(&'static str, &'static str)] {
    &[
        ("Minimal", NEUTRAL_CSS),
        ("Material", MATERIAL_CSS),
        ("Dark SaaS", NEUTRAL_CSS),
        ("Flat", NEUTRAL_CSS),
        ("Skeuomorphic", SKEUOMORPHIC_CSS),
        ("Metro", NEUTRAL_CSS),
        ("Lara", NEUTRAL_CSS),
        ("Fluent", NEUTRAL_CSS),
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
