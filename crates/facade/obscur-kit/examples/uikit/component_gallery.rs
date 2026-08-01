#![doc = include_str!("component_gallery.doc/_module.md")]

use obscur_kit::view::view::ViewNode;

use super::classed_node::{el, text};

#[doc = include_str!("component_gallery.doc/gallery.md")]
pub(super) fn gallery() -> ViewNode {
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

#[doc = include_str!("component_gallery.doc/row.md")]
fn row(variant: &str, cards: Vec<ViewNode>) -> ViewNode {
    el(&format!("row {variant}"), cards)
}

#[doc = include_str!("component_gallery.doc/card.md")]
fn card(extra: &str, label: &str, mut body: Vec<ViewNode>) -> ViewNode {
    let mut kids = vec![text("card-label", label)];
    kids.append(&mut body);
    el(&format!("card {extra}"), kids)
}

#[doc = include_str!("component_gallery.doc/buttons_card.md")]
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

#[doc = include_str!("component_gallery.doc/btn.md")]
fn btn(variant: &str, label: &str) -> ViewNode {
    el(&format!("btn {variant}"), vec![text("btn-label", label)])
}

#[doc = include_str!("component_gallery.doc/selection_card.md")]
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
    card(
        "col",
        "SELECTION",
        vec![el("sel-cols", vec![checks, radios, toggles])],
    )
}

#[doc = include_str!("component_gallery.doc/check_row.md")]
fn check_row(boxclass: &str, inner: Vec<ViewNode>, label: &str) -> ViewNode {
    el(
        "sel-item",
        vec![el(boxclass, inner), text("sel-text", label)],
    )
}

#[doc = include_str!("component_gallery.doc/radio_row.md")]
fn radio_row(on: bool, label: &str) -> ViewNode {
    let ring = if on { "radio-on" } else { "radio-off" };
    let inner = if on {
        vec![el("radio-dot", vec![])]
    } else {
        vec![]
    };
    el("sel-item", vec![el(ring, inner), text("sel-text", label)])
}

#[doc = include_str!("component_gallery.doc/toggle_row.md")]
fn toggle_row(on: bool, label: &str) -> ViewNode {
    let track = if on {
        el("toggle-on", vec![el("toggle-knob", vec![])])
    } else {
        el("toggle-off", vec![el("toggle-knob-off", vec![])])
    };
    el("sel-item", vec![track, text("sel-text", label)])
}

#[doc = include_str!("component_gallery.doc/fields_card.md")]
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
    card(
        "col",
        "TEXT FIELDS",
        vec![el("field-col", vec![filled, outlined, error])],
    )
}

#[doc = include_str!("component_gallery.doc/menu_card.md")]
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
            el(
                "chip chip-filled",
                vec![text("chip-check", "✓"), text("chip-text", "Design")],
            ),
            el("chip chip-outlined", vec![text("chip-text", "Research")]),
            el("chip chip-outlined", vec![text("chip-text", "Ops")]),
        ],
    );
    card("col", "MENU · DROPDOWN", vec![select, panel, chips])
}

#[doc = include_str!("component_gallery.doc/cardlist_card.md")]
fn cardlist_card() -> ViewNode {
    let cover = el("mini-cover", vec![text("mini-cover-text", "cover image")]);
    let body = el(
        "mini-body",
        vec![
            text("mini-title", "Quarterly report"),
            text("mini-sub", "Updated 2 days ago · 14 pages"),
            el(
                "mini-actions",
                vec![
                    btn("btn-filled btn-sm", "Open"),
                    btn("btn-text btn-sm", "Share"),
                ],
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

#[doc = include_str!("component_gallery.doc/person.md")]
fn person(avatar_class: &str, initials: &str, name: &str, email: &str) -> ViewNode {
    el(
        "person",
        vec![
            el(
                &format!("avatar {avatar_class}"),
                vec![text("avatar-text", initials)],
            ),
            el(
                "person-meta",
                vec![text("person-name", name), text("person-email", email)],
            ),
            text("person-menu", "⋮"),
        ],
    )
}

#[doc = include_str!("component_gallery.doc/navigation_card.md")]
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

#[doc = include_str!("component_gallery.doc/nav_tab.md")]
fn nav_tab(label: &str, active: bool) -> ViewNode {
    let mut kids = vec![text(
        if active {
            "nav-tab-label active"
        } else {
            "nav-tab-label"
        },
        label,
    )];
    if active {
        kids.push(el("nav-tab-underline", vec![]));
    }
    el("nav-tab", kids)
}

#[doc = include_str!("component_gallery.doc/nav_item.md")]
fn nav_item(active: bool, dot: &str, label: &str) -> ViewNode {
    el(
        if active {
            "nav-item active"
        } else {
            "nav-item"
        },
        vec![el(dot, vec![]), text("nav-item-text", label)],
    )
}

#[doc = include_str!("component_gallery.doc/badges_card.md")]
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

#[doc = include_str!("component_gallery.doc/badge.md")]
fn badge(variant: &str, label: &str) -> ViewNode {
    el(&format!("badge {variant}"), vec![text("badge-text", label)])
}

#[doc = include_str!("component_gallery.doc/stack_avatar.md")]
fn stack_avatar(class: &str, initials: &str, overlap: bool) -> ViewNode {
    let cls = if overlap {
        format!("avatar stack {class} overlap")
    } else {
        format!("avatar stack {class}")
    };
    el(&cls, vec![text("avatar-text", initials)])
}

#[doc = include_str!("component_gallery.doc/sliders_card.md")]
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
    card(
        "col",
        "SLIDERS · PROGRESS",
        vec![head, slider, progress_label, progress, spinner],
    )
}

#[doc = include_str!("component_gallery.doc/table_card.md")]
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

#[doc = include_str!("component_gallery.doc/trow.md")]
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

#[doc = include_str!("component_gallery.doc/feedback_card.md")]
fn feedback_card() -> ViewNode {
    let row = el(
        "feedback-row",
        vec![
            btn("btn-filled btn-dialog", "Open dialog"),
            btn("btn-outlined btn-wide", "Show snackbar"),
            el(
                "chip chip-tonal chip-wide",
                vec![text("chip-text", "Hover for tooltip")],
            ),
        ],
    );
    card("full feedback-c", "FEEDBACK · OVERLAYS", vec![row])
}
