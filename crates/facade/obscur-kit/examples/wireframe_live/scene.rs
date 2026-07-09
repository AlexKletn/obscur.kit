//! The **interactive Wireframe-Kit** scene — authored once, driven by both the
//! live windowed Vello example (`wireframe_live_window.rs`) and the headless
//! CPU→PNG proof (`wireframe_live_headless.rs`).
//!
//! # The point
//!
//! Every control here is **BARE**: a real [`Button`] / [`Input`] / [`Checkbox`] /
//! [`Radio`] / [`Select`] (or a raw `<a>`), carrying **no** author CSS for its
//! appearance. All of the control look — the paper fill, the 1.5px ink border, the
//! 2px radius, the hover fill shift, the dashed focus ring, the inverted `:active`,
//! the checkbox `✓` box, the radio dot, the select `▼`/`▲` — comes entirely from
//! the kit's **UA default stylesheet** ([`obscur_kit::ua_stylesheet`]), which the
//! two drivers install on the runtime. This demo therefore *proves* the UA look:
//! remove the installed default sheet and every control goes unstyled.
//!
//! The scene authors only the **LAYOUT CHROME** — the graph-paper page, the
//! centered column, the masthead card (title + mono subtitle + description + the
//! five state-legend chips), the numbered section cards, the captions, and the
//! footer — via `.wf-*` classes. It never writes a `button { … }` / `input { … }` /
//! `checkbox { … }` / `.checkbox-box { … }` / `a { … }` rule, so the UA default
//! shows through unmodified.
//!
//! Like the other shared scenes it is **backend-pure**: it names only the toolkit's
//! view / widget / primitive / CSS authoring surfaces (`obscur_kit::*`) and never a
//! renderer, window, or GPU type, so the same [`view`] + [`stylesheet`] feed a
//! software rasterizer and a GPU surface unchanged.
//!
//! # Sections (all LIVE bare controls)
//!
//! 1. **Button** `<button>` — a live [`Button`] (`:hover` fill shift, `:active`
//!    invert, `:focus` ring; a click bumps the status counter) beside a disabled one.
//! 2. **Text Field** `<input>` — a live [`Input`] (click → focus ring + caret,
//!    type → text) beside a disabled one.
//! 3. **Textarea** `<textarea>` — a live [`Input`] (there is no dedicated textarea
//!    widget; the UA `input, textarea { … }` rule styles both identically, so a
//!    bare `Input` renders the same base look — noted here, not a separate control).
//! 4. **Select** `<select>` — a live [`Select`] state machine: the header opens the
//!    list (arrow ▲, all options inline), and a click while open picks the next
//!    option and closes (arrow ▼, showing the chosen label). Per-**option**
//!    hit-routing is NOT a bare-`Select` capability (the widget bubbles one event
//!    for the whole control and always renders all options inline), so "pick" is a
//!    cycle-to-next; noted. Beside it a disabled one.
//! 5. **Checkbox** `<input type=checkbox>` — a live [`Checkbox`] (box `✓` on click),
//!    a tri-state one, and a disabled one.
//! 6. **Radio** `<input type=radio>` — a live group of three [`Radio`]s sharing one
//!    selection (the chosen one shows the filled dot), plus a disabled one.
//! 7. **Text Link** `<a>` — a bare `<a>` element showing the UA underline and the
//!    `:hover` bold.
//!
//! The purely-static spec-only groups from the source kit (range / progress / meter
//! / file / color / fieldset / table / scrollbar) are **skipped** — this demo
//! prioritises the genuinely interactive controls.
// Both examples share this module via `#[path]`; each uses a subset of it, so some
// items look dead/unreachable-pub inside a single example binary.
#![allow(dead_code, unreachable_pub)]

use obscur_kit::identity::{ComponentId, InternedSymbol};
use obscur_kit::primitives::{Container, Text};
use obscur_kit::view::view::{
    CLASS, EVENT_CLICK, ID, TYPE, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty,
    ViewValue,
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

/// The enabled push-button's click command (bumps the click counter).
pub const CMD_BTN: InternedSymbol = InternedSymbol::from_raw(0x7100_0001);
/// The **disabled** button's command — deliberately left without a handler so a
/// click resolves to a genuine no-op.
pub const CMD_BTN_OFF: InternedSymbol = InternedSymbol::from_raw(0x7100_0002);
/// The editable text field's per-keystroke text command.
pub const CMD_FIELD: InternedSymbol = InternedSymbol::from_raw(0x7100_0003);
/// The **disabled** field's command — no handler, so it is a no-op.
pub const CMD_FIELD_OFF: InternedSymbol = InternedSymbol::from_raw(0x7100_0004);
/// The textarea (a live `Input`) per-keystroke text command.
pub const CMD_AREA: InternedSymbol = InternedSymbol::from_raw(0x7100_0005);
/// The select's command (open when closed; pick-next + close when open).
pub const CMD_SEL: InternedSymbol = InternedSymbol::from_raw(0x7100_0006);
/// The **disabled** select's command — no handler, so it is a no-op.
pub const CMD_SEL_OFF: InternedSymbol = InternedSymbol::from_raw(0x7100_0007);
/// The plain checkbox's change command (toggles it).
pub const CMD_CHK: InternedSymbol = InternedSymbol::from_raw(0x7100_0008);
/// The tri-state checkbox's change command (advances mixed → checked → …).
pub const CMD_TRI: InternedSymbol = InternedSymbol::from_raw(0x7100_0009);
/// The **disabled** checkbox's command — no handler, so it is a no-op.
pub const CMD_CHK_OFF: InternedSymbol = InternedSymbol::from_raw(0x7100_000A);
/// The **disabled** radio's command — no handler, so it is a no-op.
pub const CMD_RADIO_OFF: InternedSymbol = InternedSymbol::from_raw(0x7100_000B);
/// The text link's click command (updates the status line).
pub const CMD_LINK: InternedSymbol = InternedSymbol::from_raw(0x7100_000C);

/// The number of radio options in the group.
pub const RADIO_COUNT: usize = 3;
/// The base command for the radio group; option `i` fires `CMD_RADIO_BASE + i`.
pub const CMD_RADIO_BASE: u32 = 0x7100_0020;
/// The number of select options.
pub const SELECT_COUNT: usize = 3;

/// The command a radio option at `index` fires (on `EVENT_CHANGE`).
#[must_use]
pub fn radio_command(index: usize) -> InternedSymbol {
    InternedSymbol::from_raw(CMD_RADIO_BASE + index as u32)
}

/// The human labels of the radio options.
pub const RADIO_LABELS: [&str; RADIO_COUNT] = ["Small", "Medium", "Large"];
/// The human labels of the select options. Single tokens on purpose: a BARE
/// `Select` is `display: flex` (label + arrow), and under this engine its flexible
/// label item resolves to a narrow intrinsic width, so a two-word label ("Option
/// one") wraps and spills below the box. Single-word options render the field
/// cleanly with no restyling (the whole point — no author CSS on the control).
pub const SELECT_LABELS: [&str; SELECT_COUNT] = ["One", "Two", "Three"];
/// The closed-state prompt shown before anything is picked (a single token, so it
/// never wraps — see [`SELECT_LABELS`]).
pub const SELECT_PROMPT: &str = "Choose…";

/// The `<a>` element's stable component id (a raw semantic element the demo emits
/// directly — the widgets crate has no anchor widget; the UA `a { … }` rule styles
/// it purely by its CSS type).
const ANCHOR_ID: ComponentId = ComponentId::from_raw(0x7100_00A0);

// ---------------------------------------------------------------------------
// The shared interaction state.
// ---------------------------------------------------------------------------

/// The live interaction state the view is parameterized by. Both drivers own one
/// behind an `Rc<RefCell<_>>`: the command / text-input handlers mutate it, and the
/// reactive [`view`] builder reads it back to rebuild every frame.
#[derive(Debug, Clone)]
pub struct WireState {
    /// How many times the enabled button has been clicked.
    pub clicks: u32,
    /// A short human description of the last interaction (status line).
    pub last_event: String,
    /// The editable text field's current text, mirrored from the editing session.
    pub field: String,
    /// The textarea's current text (a live `Input`), mirrored from editing.
    pub area: String,
    /// The plain checkbox (real widget state machine).
    pub check: Checkbox,
    /// The tri-state checkbox (starts indeterminate).
    pub tri: Checkbox,
    /// The radio group's selected index.
    pub radio: usize,
    /// The dropdown (real widget: open/selected state machine).
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

/// The layout-chrome stylesheet: masthead, section cards, captions, the legend
/// swatches, and the graph-paper page. Returned as `&'static str` so both drivers
/// compile it through the real `obscur-css` frontend.
///
/// Palette matches the source kit: ink `#2b2a27`, muted `#5a584f`, faint `#9b998f`,
/// disabled-ink `#b7b5ad`, page `#eceae5`, surface `#faf9f6`, hover `#e4e2db`,
/// disabled-fill `#f2f0ea`. Fonts are the host's (no IBM Plex), so only sizes and
/// weights are set — mono captions/subtitles read in the host font by design.
#[must_use]
pub fn stylesheet() -> &'static str {
    "\
* { box-sizing: border-box; }\n\
\
/* Graph-paper page: flat #eceae5 plus two crossing 1px grid layers at 24px (a real\n\
   grid on Vello; flat first-stop on the CPU renderer, by policy). */\n\
#app { display: flex; flex-direction: column; align-items: center; width: 100%; \
       padding: 48px 40px 64px; background-color: #eceae5; \
       background-image: \
         repeating-linear-gradient(#e2e0d9 0 1px, transparent 1px 24px), \
         repeating-linear-gradient(90deg, #e2e0d9 0 1px, transparent 1px 24px); }\n\
.wf-col { display: flex; flex-direction: column; gap: 24px; width: 1120px; flex-shrink: 0; }\n\
\
/* ---- Masthead card ---- */\n\
.wf-masthead { display: flex; flex-direction: column; gap: 16px; padding: 32px 34px; flex-shrink: 0; \
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
.wf-card { display: flex; flex-direction: column; gap: 24px; padding: 28px 30px 32px; flex-shrink: 0; \
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
/* ---- Footer ---- */\n\
.wf-footer { display: flex; flex-direction: row; justify-content: space-between; \
    padding: 8px 4px 0; }\n\
.wf-foot-line { height: 16px; color: #9b998f; font-size: 11px; font-weight: 500; }\n\
"
}

// ---------------------------------------------------------------------------
// The view builder.
// ---------------------------------------------------------------------------

/// Builds the full wireframe view tree from `state`: the graph-paper page holding
/// a centered column of the masthead, the seven live-control section cards, and the
/// footer. Pure view/widget/primitive construction — no backend types — so both
/// drivers call it identically. The controls are BARE; only the UA sheet styles
/// them.
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
            section("05", "Checkbox", "<input type=checkbox>", checkbox_body(state)),
            section("06", "Radio", "<input type=radio>", radio_body(state)),
            section("07", "Text Link", "<a>", link_body()),
            footer(),
        ],
    );

    let app = el("wf-page", vec![column])
        .with_prop(ViewProperty::new(ID, ViewValue::Text("app".into())));
    View::single(app)
}

/// The masthead card: title + mono subtitle, the description paragraph, the live
/// status line, and the five state-legend chips.
fn masthead(state: &WireState) -> ViewNode {
    let title_row = el(
        "wf-mast-row",
        vec![
            text("wf-mast-title", "Wireframe UI Kit"),
            text("wf-mast-sub", "standard browser elements · all visual states"),
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

/// One legend chip: a state swatch beside its mono label.
fn chip(label: &str, swatch_variant: &str) -> ViewNode {
    let swatch = el(&format!("wf-sw {swatch_variant}"), vec![]);
    el("wf-chip", vec![swatch, text("wf-chip-label", label)])
}

/// The footer: two mono lines at opposite ends.
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

/// 01 · Button: a live button beside a disabled one.
fn button_body() -> ViewNode {
    body(vec![
        slot("DEFAULT · LIVE", control("btn", Button::new("Button", CMD_BTN).view())),
        slot(
            "DISABLED",
            control("btn-off", Button::new("Button", CMD_BTN_OFF).disabled(true).view()),
        ),
    ])
}

/// 02 · Text Field: a live input beside a disabled one.
fn field_body(state: &WireState) -> ViewNode {
    body(vec![
        slot(
            "DEFAULT · LIVE",
            control(
                "field",
                Input::with_value(CMD_FIELD, state.field.clone())
                    .placeholder("Type here…")
                    .view(),
            ),
        ),
        slot(
            "DISABLED",
            control(
                "field-off",
                Input::with_value(CMD_FIELD_OFF, "read only").disabled(true).view(),
            ),
        ),
    ])
}

/// 03 · Textarea: a live `Input` (no dedicated textarea widget — the UA
/// `input, textarea` rule styles both identically).
fn area_body(state: &WireState) -> ViewNode {
    body(vec![slot(
        "MULTILINE · LIVE",
        control(
            "area",
            Input::with_value(CMD_AREA, state.area.clone())
                .placeholder("Longer entered text…")
                .view(),
        ),
    )])
}

/// 04 · Select: a live dropdown beside a disabled one.
fn select_body(state: &WireState) -> ViewNode {
    body(vec![
        slot("DEFAULT · LIVE", control("select", select_widget(state))),
        slot(
            "DISABLED",
            control(
                "select-off",
                Select::new(CMD_SEL_OFF).with_options([SELECT_PROMPT]).disabled(true).view(),
            ),
        ),
    ])
}

/// The live select's bare view: while open, the real widget renders all options
/// inline with the ▲ arrow; while closed, a proxy `Select` shows just the current
/// label (or the prompt) with the ▼ arrow. Both fire [`CMD_SEL`], so a click opens
/// (when closed) or picks-next + closes (when open) via the single handler.
fn select_widget(state: &WireState) -> View {
    if state.select.is_open() {
        state.select.view()
    } else {
        let label = state.select.selected_label().unwrap_or(SELECT_PROMPT);
        Select::new(CMD_SEL).with_options([label]).view()
    }
}

/// 05 · Checkbox: a live one, a tri-state one, and a disabled one.
fn checkbox_body(state: &WireState) -> ViewNode {
    body(vec![
        slot("LIVE", control("chk", state.check.view())),
        slot("TRI-STATE", control("chk-tri", state.tri.view())),
        slot(
            "DISABLED",
            control(
                "chk-off",
                Checkbox::new("Disabled", CMD_CHK_OFF).checked(true).disabled(true).view(),
            ),
        ),
    ])
}

/// 06 · Radio: a live group of three sharing one selection, plus a disabled one.
fn radio_body(state: &WireState) -> ViewNode {
    let mut slots: Vec<ViewNode> = (0..RADIO_COUNT)
        .map(|i| {
            let selected = state.radio == i;
            slot(
                if selected { "SELECTED" } else { "OPTION" },
                control(
                    &format!("radio-{i}"),
                    Radio::new(RADIO_LABELS[i], radio_command(i)).selected(selected).view(),
                ),
            )
        })
        .collect();
    slots.push(slot(
        "DISABLED",
        control("radio-off", Radio::new("Disabled", CMD_RADIO_OFF).disabled(true).view()),
    ));
    body(slots)
}

/// 07 · Text Link: a bare `<a>` element (UA underline + `:hover` bold).
fn link_body() -> ViewNode {
    body(vec![slot("DEFAULT · LIVE", anchor("link", "Text link"))])
}

// ---------------------------------------------------------------------------
// Small view helpers.
// ---------------------------------------------------------------------------

/// A numbered section card: the mono `NN` + `<h2>` title + `<tag>` code header
/// (with a bottom rule) over the group body.
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

/// A `.wf-body` flex row holding the section's control slots.
fn body(slots: Vec<ViewNode>) -> ViewNode {
    el("wf-body", slots)
}

/// One control slot: the bare control above a mono caption.
fn slot(caption: &str, control: ViewNode) -> ViewNode {
    el("wf-slot", vec![control, text("wf-cap", caption)])
}

/// Tags a bare widget's rendered root node with a unique `id` (so the headless
/// proof can hit-test it by identity). It carries **no** class, so nothing but the
/// UA sheet styles it.
fn control(id: &str, view: View) -> ViewNode {
    first_node(view).with_prop(ViewProperty::new(ID, ViewValue::Text(id.into())))
}

/// A bare `<a>` anchor element: a semantic element carrying the stable CSS type
/// `a` (so the UA `a { … }` / `a:hover { … }` rules match), a click binding, a
/// unique `id`, and the link text as its child.
fn anchor(id: &str, label: &str) -> ViewNode {
    ViewNode::element(ANCHOR_ID)
        .with_prop(ViewProperty::new(TYPE, ViewValue::from("a")))
        .with_prop(ViewProperty::new(ID, ViewValue::Text(id.into())))
        .with_event(ViewEventBinding::new(EVENT_CLICK, CMD_LINK))
        .with_children(ViewFragment::from_nodes(vec![text_inline(label)]))
}

/// Wraps `children` in a classed [`Container`] element node.
fn el(class: &str, children: Vec<ViewNode>) -> ViewNode {
    Container::new()
        .with_children(children)
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}

/// A classed [`Text`] node (the class carries colour / size / weight).
fn text(class: &str, content: &str) -> ViewNode {
    Text::new(content.to_owned())
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}

/// A bare (class-less) [`Text`] node used as a control's / anchor's label child.
fn text_inline(content: &str) -> ViewNode {
    Text::new(content.to_owned()).into_view_node()
}

/// Pulls a widget's first rendered [`ViewNode`] out of the [`View`] it produces.
fn first_node(view: View) -> ViewNode {
    view.into_fragment().nodes().to_vec().remove(0)
}
