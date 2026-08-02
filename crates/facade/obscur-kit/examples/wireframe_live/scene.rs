#![doc = include_str!("scene.doc/_module.md")]
// Both examples share this module via `#[path]`; each uses a subset of it, so some
// items look dead/unreachable-pub inside a single example binary.
#![allow(dead_code, unreachable_pub)]

use obscur_kit::identity::{ComponentId, InternedSymbol};
use obscur_kit::primitives::{Container, Text};
use obscur_kit::view::view::{
    CLASS, EVENT_CLICK, ID, LOCAL_NAME, View, ViewEventBinding, ViewFragment, ViewNode,
    ViewProperty, ViewValue,
};
use obscur_kit::widgets::button::Button;
use obscur_kit::widgets::checkbox::Checkbox;
use obscur_kit::widgets::input::Input;
use obscur_kit::widgets::radio::Radio;
use obscur_kit::widgets::select::Select;

// ---------------------------------------------------------------------------
// Command symbols the interactive controls fire. `pub` so BOTH drivers import
// the exact same values and register matching handlers (spec §2.3): the runtime
// only compares symbols, so the app-owned command identity must agree.
// ---------------------------------------------------------------------------

#[doc = include_str!("scene.doc/CMD_BTN.md")]
pub const CMD_BTN: InternedSymbol = InternedSymbol::from_raw(0x7100_0001);
#[doc = include_str!("scene.doc/CMD_BTN_OFF.md")]
pub const CMD_BTN_OFF: InternedSymbol = InternedSymbol::from_raw(0x7100_0002);
#[doc = include_str!("scene.doc/CMD_FIELD.md")]
pub const CMD_FIELD: InternedSymbol = InternedSymbol::from_raw(0x7100_0003);
#[doc = include_str!("scene.doc/CMD_FIELD_OFF.md")]
pub const CMD_FIELD_OFF: InternedSymbol = InternedSymbol::from_raw(0x7100_0004);
#[doc = include_str!("scene.doc/CMD_AREA.md")]
pub const CMD_AREA: InternedSymbol = InternedSymbol::from_raw(0x7100_0005);
#[doc = include_str!("scene.doc/CMD_SEL.md")]
pub const CMD_SEL: InternedSymbol = InternedSymbol::from_raw(0x7100_0006);
#[doc = include_str!("scene.doc/CMD_SEL_OFF.md")]
pub const CMD_SEL_OFF: InternedSymbol = InternedSymbol::from_raw(0x7100_0007);
#[doc = include_str!("scene.doc/CMD_CHK.md")]
pub const CMD_CHK: InternedSymbol = InternedSymbol::from_raw(0x7100_0008);
#[doc = include_str!("scene.doc/CMD_TRI.md")]
pub const CMD_TRI: InternedSymbol = InternedSymbol::from_raw(0x7100_0009);
#[doc = include_str!("scene.doc/CMD_CHK_OFF.md")]
pub const CMD_CHK_OFF: InternedSymbol = InternedSymbol::from_raw(0x7100_000A);
#[doc = include_str!("scene.doc/CMD_RADIO_OFF.md")]
pub const CMD_RADIO_OFF: InternedSymbol = InternedSymbol::from_raw(0x7100_000B);
#[doc = include_str!("scene.doc/CMD_LINK.md")]
pub const CMD_LINK: InternedSymbol = InternedSymbol::from_raw(0x7100_000C);

#[doc = include_str!("scene.doc/RADIO_COUNT.md")]
pub const RADIO_COUNT: usize = 3;
#[doc = include_str!("scene.doc/CMD_RADIO_BASE.md")]
pub const CMD_RADIO_BASE: u32 = 0x7100_0020;
#[doc = include_str!("scene.doc/SELECT_COUNT.md")]
pub const SELECT_COUNT: usize = 3;

#[doc = include_str!("scene.doc/radio_command.md")]
#[must_use]
pub fn radio_command(index: usize) -> InternedSymbol {
    InternedSymbol::from_raw(CMD_RADIO_BASE + index as u32)
}

#[doc = include_str!("scene.doc/RADIO_LABELS.md")]
pub const RADIO_LABELS: [&str; RADIO_COUNT] = ["Small", "Medium", "Large"];
#[doc = include_str!("scene.doc/SELECT_LABELS.md")]
pub const SELECT_LABELS: [&str; SELECT_COUNT] = ["One", "Two", "Three"];
#[doc = include_str!("scene.doc/SELECT_PROMPT.md")]
pub const SELECT_PROMPT: &str = "Choose…";

#[doc = include_str!("scene.doc/ANCHOR_ID.md")]
const ANCHOR_ID: ComponentId = ComponentId::from_raw(0x7100_00A0);

// ---------------------------------------------------------------------------
// The shared interaction state.
// ---------------------------------------------------------------------------

#[doc = include_str!("scene.doc/WireState.md")]
#[derive(Debug, Clone)]
pub struct WireState {
    #[doc = include_str!("scene.doc/clicks.md")]
    pub clicks: u32,
    #[doc = include_str!("scene.doc/last_event.md")]
    pub last_event: String,
    #[doc = include_str!("scene.doc/field.md")]
    pub field: String,
    #[doc = include_str!("scene.doc/area.md")]
    pub area: String,
    #[doc = include_str!("scene.doc/check.md")]
    pub check: Checkbox,
    #[doc = include_str!("scene.doc/tri.md")]
    pub tri: Checkbox,
    #[doc = include_str!("scene.doc/radio.md")]
    pub radio: usize,
    #[doc = include_str!("scene.doc/select.md")]
    pub select: Select,
}

impl Default for WireState {
    fn default() -> Self {
        Self {
            clicks: 0,
            last_event: "ready".to_owned(),
            field: String::new(),
            area: String::new(),
            check: Checkbox::new("Checkbox with adjacent label", CMD_CHK),
            tri: Checkbox::indeterminate("Mixed", CMD_TRI),
            radio: 0,
            select: Select::new(CMD_SEL).with_options(SELECT_LABELS),
        }
    }
}

// ---------------------------------------------------------------------------
// The stylesheet — LAYOUT CHROME ONLY.
//
// Every rule keys a `.wf-*` layout class or the `#app` page root. There is NO
// rule for any control type (`button` / `input` / `checkbox` / `radio` / `select`
// / `a`) or any widget sub-part (`.checkbox-box` / `.radio-dot` / `.select-arrow`),
// so the installed UA default sheet is what gives the bare controls their look —
// that is the whole demonstration. The `.wf-sw-*` legend swatches are decorative
// masthead chips, NOT controls.
// ---------------------------------------------------------------------------

#[doc = include_str!("scene.doc/stylesheet.md")]
#[must_use]
pub fn stylesheet() -> &'static str {
    "\
* { box-sizing: border-box; }\n\
\
/* Graph-paper page: flat #eceae5 plus two crossing 1px grid layers at 24px (a real\n\
   grid on Vello; flat first-stop on the CPU renderer, by policy). */\n\
#app { display: flex; flex-direction: column; align-items: center; width: 100%; \
       height: 100%; overflow: auto; \
       padding: 48px 40px 64px; background-color: #eceae5; \
       background-image: \
         repeating-linear-gradient(#e2e0d9 0 1px, transparent 1px 24px), \
         repeating-linear-gradient(90deg, #e2e0d9 0 1px, transparent 1px 24px); }\n\
.wf-col { display: flex; flex-direction: column; gap: 24px; width: 1120px; }\n\
\
/* ---- Masthead card ---- */\n\
.wf-masthead { display: flex; flex-direction: column; gap: 16px; padding: 32px 34px; \
    background-color: #faf9f6; border: 1.5px solid #2b2a27; border-radius: 4px; }\n\
.wf-mast-row { display: flex; flex-direction: row; align-items: flex-end; gap: 16px; }\n\
.wf-mast-title { height: 38px; color: #2b2a27; font-size: 30px; font-weight: 700; }\n\
.wf-mast-sub { height: 18px; color: #9b998f; font-size: 13px; font-weight: 500; }\n\
.wf-mast-desc { width: 640px; height: 46px; color: #5a584f; font-size: 14px; }\n\
.wf-status { height: 22px; color: #5a584f; font-size: 13px; font-weight: 600; \
    padding: 5px 10px; background-color: #f2f0ea; border-radius: 3px; }\n\
\
/* ---- Legend chips ---- */\n\
.wf-legend { display: flex; flex-direction: row; gap: 18px; }\n\
.wf-chip { display: flex; flex-direction: row; align-items: center; gap: 8px; }\n\
.wf-chip-label { height: 14px; color: #5a584f; font-size: 10px; font-weight: 600; }\n\
.wf-sw { width: 26px; height: 15px; border-radius: 2px; }\n\
.wf-sw-default { border: 1.5px solid #2b2a27; background-color: #faf9f6; }\n\
.wf-sw-hover { border: 1.5px solid #2b2a27; background-color: #e4e2db; }\n\
.wf-sw-focus { border: 1.5px solid #2b2a27; background-color: #faf9f6; \
    outline: 2px dashed #2b2a27; outline-offset: 2px; }\n\
.wf-sw-active { border: 1.5px solid #2b2a27; background-color: #2b2a27; }\n\
.wf-sw-disabled { border: 1.5px dashed #b7b5ad; background-color: #f2f0ea; }\n\
\
/* ---- Section card + header ---- */\n\
.wf-card { display: flex; flex-direction: column; gap: 24px; padding: 28px 30px 32px; \
    background-color: #faf9f6; border: 1.5px solid #2b2a27; border-radius: 4px; }\n\
.wf-head { display: flex; flex-direction: row; align-items: flex-end; gap: 14px; \
    padding-bottom: 12px; border-bottom: 1.5px solid #2b2a27; }\n\
.wf-num { height: 20px; color: #2b2a27; font-size: 13px; font-weight: 700; }\n\
.wf-h2 { height: 24px; color: #2b2a27; font-size: 19px; font-weight: 700; }\n\
.wf-tag { height: 20px; color: #9b998f; font-size: 12px; font-weight: 500; }\n\
.wf-body { display: flex; flex-direction: row; align-items: flex-start; gap: 34px; }\n\
.wf-slot { display: flex; flex-direction: column; align-items: flex-start; gap: 12px; }\n\
.wf-cap { height: 14px; color: #9b998f; font-size: 10px; font-weight: 600; }\n\
\
/* ---- Author-level field SIZING (a real form sizes its fields; the LOOK still\n\
   comes from the UA sheet). The text field is single-line (UA `input` nowrap) at a\n\
   definite width; the textarea is a live multi-line `Input` (type `textarea`), so\n\
   the UA `textarea` rule already gives it `white-space: pre-wrap` — this class only\n\
   sets its definite `360×96` box so it wraps inside it. ---- */\n\
.wf-field { width: 220px; overflow: hidden; }\n\
.wf-area { width: 360px; height: 96px; overflow: hidden; }\n\
.wf-select { width: 200px; }\n\
\
/* ---- Footer ---- */\n\
.wf-footer { display: flex; flex-direction: row; justify-content: space-between; \
    padding: 8px 4px 0; }\n\
.wf-foot-line { height: 16px; color: #9b998f; font-size: 11px; font-weight: 500; }\n\
"
}

// ---------------------------------------------------------------------------
// The view builder.
// ---------------------------------------------------------------------------

#[doc = include_str!("scene.doc/view.md")]
#[must_use]
pub fn view(state: &WireState) -> View {
    let column = el(
        "wf-col",
        vec![
            masthead(state),
            section("01", "Button", "<button>", button_body()),
            section("02", "Text Field", "<input>", field_body(state)),
            section("03", "Textarea", "<textarea>", area_body(state)),
            section("04", "Select", "<select>", select_body(state)),
            section(
                "05",
                "Checkbox",
                "<input type=checkbox>",
                checkbox_body(state),
            ),
            section("06", "Radio", "<input type=radio>", radio_body(state)),
            section("07", "Text Link", "<a>", link_body()),
            footer(),
        ],
    );

    let app =
        el("wf-page", vec![column]).with_prop(ViewProperty::new(ID, ViewValue::Text("app".into())));
    View::single(app)
}

#[doc = include_str!("scene.doc/masthead.md")]
fn masthead(state: &WireState) -> ViewNode {
    let title_row = el(
        "wf-mast-row",
        vec![
            text("wf-mast-title", "Wireframe UI Kit"),
            text(
                "wf-mast-sub",
                "standard browser elements · all visual states",
            ),
        ],
    );
    let desc = text(
        "wf-mast-desc",
        "A grayscale specification sheet of native HTML controls. Every control is \
         shown across its full range of interaction states. Hovers use a fill shift \
         only — the cursor stays as an arrow, matching desktop pointer behaviour.",
    );
    let status = el(
        "wf-status",
        vec![text_inline(&format!(
            "last event: {}   ·   clicks: {}",
            state.last_event, state.clicks
        ))],
    );
    let legend = el(
        "wf-legend",
        vec![
            chip("DEFAULT", "wf-sw-default"),
            chip("HOVER", "wf-sw-hover"),
            chip("FOCUS", "wf-sw-focus"),
            chip("ACTIVE", "wf-sw-active"),
            chip("DISABLED", "wf-sw-disabled"),
        ],
    );
    el("wf-masthead", vec![title_row, desc, status, legend])
}

#[doc = include_str!("scene.doc/chip.md")]
fn chip(label: &str, swatch_variant: &str) -> ViewNode {
    let swatch = el(&format!("wf-sw {swatch_variant}"), vec![]);
    el("wf-chip", vec![swatch, text("wf-chip-label", label)])
}

#[doc = include_str!("scene.doc/footer.md")]
fn footer() -> ViewNode {
    el(
        "wf-footer",
        vec![
            text("wf-foot-line", "WIREFRAME UI KIT"),
            text("wf-foot-line", "GRAYSCALE"),
        ],
    )
}

// ---------------------------------------------------------------------------
// Section bodies — each holds one or more BARE controls.
// ---------------------------------------------------------------------------

#[doc = include_str!("scene.doc/button_body.md")]
fn button_body() -> ViewNode {
    body(vec![
        slot(
            "DEFAULT · LIVE",
            control("btn", Button::new("Button", CMD_BTN).view()),
        ),
        slot(
            "DISABLED",
            control(
                "btn-off",
                Button::new("Button", CMD_BTN_OFF).disabled(true).view(),
            ),
        ),
    ])
}

#[doc = include_str!("scene.doc/field_body.md")]
fn field_body(state: &WireState) -> ViewNode {
    body(vec![
        slot(
            "DEFAULT · LIVE",
            control_sized(
                "field",
                "wf-field",
                Input::with_value(CMD_FIELD, state.field.clone())
                    .placeholder("Type here…")
                    .view(),
            ),
        ),
        slot(
            "READ ONLY",
            control_sized(
                "field-off",
                "wf-field",
                Input::with_value(CMD_FIELD_OFF, "read only")
                    .readonly(true)
                    .view(),
            ),
        ),
    ])
}

#[doc = include_str!("scene.doc/area_body.md")]
fn area_body(state: &WireState) -> ViewNode {
    body(vec![slot(
        "MULTILINE · LIVE",
        control_sized(
            "area",
            "wf-area",
            Input::with_value(CMD_AREA, state.area.clone())
                .multiline(true)
                .placeholder("Longer entered text…")
                .view(),
        ),
    )])
}

#[doc = include_str!("scene.doc/select_body.md")]
fn select_body(state: &WireState) -> ViewNode {
    body(vec![
        slot(
            "DEFAULT · LIVE",
            control_sized("select", "wf-select", select_widget(state)),
        ),
        slot(
            "DISABLED",
            control_sized(
                "select-off",
                "wf-select",
                Select::new(CMD_SEL_OFF)
                    .with_options([SELECT_PROMPT])
                    .disabled(true)
                    .view(),
            ),
        ),
    ])
}

#[doc = include_str!("scene.doc/select_widget.md")]
fn select_widget(state: &WireState) -> View {
    state.select.view()
}

#[doc = include_str!("scene.doc/checkbox_body.md")]
fn checkbox_body(state: &WireState) -> ViewNode {
    body(vec![
        slot("LIVE", control("chk", state.check.view())),
        slot("TRI-STATE", control("chk-tri", state.tri.view())),
        slot(
            "DISABLED",
            control(
                "chk-off",
                Checkbox::new("Disabled", CMD_CHK_OFF)
                    .checked(true)
                    .disabled(true)
                    .view(),
            ),
        ),
    ])
}

#[doc = include_str!("scene.doc/radio_body.md")]
fn radio_body(state: &WireState) -> ViewNode {
    let mut slots: Vec<ViewNode> = (0..RADIO_COUNT)
        .map(|i| {
            let selected = state.radio == i;
            slot(
                if selected { "SELECTED" } else { "OPTION" },
                control(
                    &format!("radio-{i}"),
                    Radio::new(RADIO_LABELS[i], radio_command(i))
                        .selected(selected)
                        .view(),
                ),
            )
        })
        .collect();
    slots.push(slot(
        "DISABLED",
        control(
            "radio-off",
            Radio::new("Disabled", CMD_RADIO_OFF).disabled(true).view(),
        ),
    ));
    body(slots)
}

#[doc = include_str!("scene.doc/link_body.md")]
fn link_body() -> ViewNode {
    body(vec![slot("DEFAULT · LIVE", anchor("link", "Text link"))])
}

// ---------------------------------------------------------------------------
// Small view helpers.
// ---------------------------------------------------------------------------

#[doc = include_str!("scene.doc/section.md")]
fn section(num: &str, title: &str, tag: &str, body_node: ViewNode) -> ViewNode {
    let head = el(
        "wf-head",
        vec![
            text("wf-num", num),
            text("wf-h2", title),
            text("wf-tag", tag),
        ],
    );
    el("wf-card", vec![head, body_node])
}

#[doc = include_str!("scene.doc/body.md")]
fn body(slots: Vec<ViewNode>) -> ViewNode {
    el("wf-body", slots)
}

#[doc = include_str!("scene.doc/slot.md")]
fn slot(caption: &str, control: ViewNode) -> ViewNode {
    el("wf-slot", vec![control, text("wf-cap", caption)])
}

#[doc = include_str!("scene.doc/control.md")]
fn control(id: &str, view: View) -> ViewNode {
    first_node(view).with_prop(ViewProperty::new(ID, ViewValue::Text(id.into())))
}

#[doc = include_str!("scene.doc/control_sized.md")]
fn control_sized(id: &str, class: &str, view: View) -> ViewNode {
    first_node(view)
        .with_prop(ViewProperty::new(ID, ViewValue::Text(id.into())))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}

#[doc = include_str!("scene.doc/anchor.md")]
fn anchor(id: &str, label: &str) -> ViewNode {
    ViewNode::element(ANCHOR_ID)
        .with_prop(ViewProperty::new(LOCAL_NAME, ViewValue::from("a")))
        .with_prop(ViewProperty::new(ID, ViewValue::Text(id.into())))
        .with_event(ViewEventBinding::new(EVENT_CLICK, CMD_LINK))
        .with_children(ViewFragment::from_nodes(vec![text_inline(label)]))
}

#[doc = include_str!("scene.doc/el.md")]
fn el(class: &str, children: Vec<ViewNode>) -> ViewNode {
    Container::new()
        .with_children(children)
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}

#[doc = include_str!("scene.doc/text.md")]
fn text(class: &str, content: &str) -> ViewNode {
    Text::new(content.to_owned())
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}

#[doc = include_str!("scene.doc/text_inline.md")]
fn text_inline(content: &str) -> ViewNode {
    Text::new(content.to_owned()).into_view_node()
}

#[doc = include_str!("scene.doc/first_node.md")]
fn first_node(view: View) -> ViewNode {
    view.into_fragment().nodes().to_vec().remove(0)
}
