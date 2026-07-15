//! The **shared interactive control-gallery** scene, authored once and driven by
//! both the live windowed Vello example (`controls_window.rs`) and the headless
//! CPU→PNG proof (`controls_headless.rs`).
//!
//! Like the capability showcase, it is deliberately **backend-pure**: it names
//! only the toolkit's view / widget / primitive / CSS authoring surfaces
//! (`obscur_kit::*`) and never a renderer, window, or GPU type, so the exact same
//! [`view`] + [`stylesheet`] feed a software rasterizer and a GPU surface
//! unchanged.
//!
//! # What it exercises
//!
//! A live gallery of the base HTML controls, each wired to real interaction state
//! so hovering, clicking, focusing, typing, toggling, and opening the dropdown
//! drive live CSS pseudo-classes recomputed by the runtime from the **real** input
//! pipeline:
//!
//! - **Button** — `EVENT_CLICK` bumps a click counter; `:hover` / `:active` /
//!   `:focus` come straight from the runtime's pointer/focus/press targets.
//! - **Checkbox** — a plain [`Checkbox`] plus a **tri-state** one; a click runs
//!   [`Checkbox::toggle`], and the reserved `CHECKED` attribute drives `:checked`.
//! - **Radio group** — composed in the demo (there is no radio widget): a set of
//!   clickable elements sharing one selected index in state, the chosen one
//!   carrying the reserved `CHECKED` attribute so `:checked` fills it.
//! - **Select** — a real [`Select`] state machine ([`toggle_open`](Select::toggle_open)
//!   / [`select`](Select::select) / [`is_open`](Select::is_open)); the header
//!   toggles the popup and, while open, the option rows render from the widget's
//!   options. Because the widget emits a single bubbled `EVENT_CLICK`, the demo
//!   attaches a per-option click command so a windowed hit-test can tell which
//!   option was chosen — noted as composed, not a widget capability.
//! - **Input** — an editable [`Input`]; a click seats the caret + `:focus` ring,
//!   and typing / IME flows through the runtime's editing session, whose
//!   per-keystroke handler mirrors the text back into state.
//! - A **disabled** variant of each control shows `:disabled` (muted + dashed) and
//!   is a genuine no-op: its `EVENT_CLICK` binds a command with **no** registered
//!   handler, so the runtime's activation resolves to nothing.
//!
//! # Open state
//!
//! There is no reserved `OPEN` attribute / `:open` pseudo-class in the engine, so
//! the select's open state is expressed **structurally**: the option rows are in
//! the tree only while the real [`Select::is_open`] is true. Toggling the popup is
//! therefore a genuine state change driven by the widget's own state machine.
// Both examples share this module via `#[path]`; each uses a subset of it (so some
// items look dead/unreachable-pub inside a single example binary, but are part of
// the shared surface the sibling example also consumes).
#![allow(dead_code, unreachable_pub)]

use obscur_kit::identity::InternedSymbol;
use obscur_kit::primitives::{Container, Text};
use obscur_kit::view::view::{
    CHECKED, CLASS, DISABLED, EVENT_CLICK, ID, View, ViewEventBinding, ViewNode, ViewProperty,
    ViewValue,
};
use obscur_kit::widgets::button::Button;
use obscur_kit::widgets::checkbox::Checkbox;
use obscur_kit::widgets::input::Input;
use obscur_kit::widgets::select::Select;

// ---------------------------------------------------------------------------
// Command symbols the interactive controls fire. Made `pub` so BOTH drivers
// import the exact same values and register matching handlers (spec §2.3): the
// runtime only compares symbols, so the app-owned command identity must agree.
// ---------------------------------------------------------------------------

/// The enabled push-button's click command (bumps the click counter).
pub const CMD_BTN: InternedSymbol = InternedSymbol::from_raw(0x7000_0001);
/// The **disabled** button's command — deliberately left without a handler so a
/// click resolves to a genuine no-op.
pub const CMD_BTN_OFF: InternedSymbol = InternedSymbol::from_raw(0x7000_0002);
/// The plain checkbox's change command (toggles it).
pub const CMD_CHK: InternedSymbol = InternedSymbol::from_raw(0x7000_0003);
/// The tri-state checkbox's change command (advances mixed → checked → …).
pub const CMD_TRI: InternedSymbol = InternedSymbol::from_raw(0x7000_0004);
/// The **disabled** checkbox's command — no handler, so it is a no-op.
pub const CMD_CHK_OFF: InternedSymbol = InternedSymbol::from_raw(0x7000_0005);
/// The select header's command (opens/closes the popup).
pub const CMD_SEL_TOGGLE: InternedSymbol = InternedSymbol::from_raw(0x7000_0006);
/// The **disabled** select header's command — no handler, so it is a no-op.
pub const CMD_SEL_OFF: InternedSymbol = InternedSymbol::from_raw(0x7000_0007);
/// The editable input's per-keystroke text command.
pub const CMD_FIELD: InternedSymbol = InternedSymbol::from_raw(0x7000_0008);
/// The **disabled** input's command — no handler, so it is a no-op.
pub const CMD_FIELD_OFF: InternedSymbol = InternedSymbol::from_raw(0x7000_0009);

/// The number of enabled radio options.
pub const RADIO_COUNT: usize = 3;
/// The base command for the composed radio group; option `i` fires `CMD_RADIO_BASE + i`.
pub const CMD_RADIO_BASE: u32 = 0x7000_0010;
/// The **disabled** radio's command — no handler, so it is a no-op.
pub const CMD_RADIO_OFF: InternedSymbol = InternedSymbol::from_raw(0x7000_001F);

/// The number of select options.
pub const SELECT_COUNT: usize = 3;
/// The base command for a composed select option; option `i` fires `CMD_SEL_OPT_BASE + i`.
pub const CMD_SEL_OPT_BASE: u32 = 0x7000_0020;

/// The command a radio option at `index` fires.
#[must_use]
pub fn radio_command(index: usize) -> InternedSymbol {
    InternedSymbol::from_raw(CMD_RADIO_BASE + index as u32)
}

/// The command a select option at `index` fires.
#[must_use]
pub fn select_option_command(index: usize) -> InternedSymbol {
    InternedSymbol::from_raw(CMD_SEL_OPT_BASE + index as u32)
}

/// The human labels of the radio options.
pub const RADIO_LABELS: [&str; RADIO_COUNT] = ["Small", "Medium", "Large"];
/// The human labels of the select options.
pub const SELECT_LABELS: [&str; SELECT_COUNT] = ["Apple", "Banana", "Cherry"];

// ---------------------------------------------------------------------------
// The shared interaction state.
// ---------------------------------------------------------------------------

/// The live interaction state the gallery view is parameterized by. Both drivers
/// own one behind an `Rc<RefCell<_>>`: the command / text-input handlers mutate
/// it, and the reactive [`view`] builder reads it back to rebuild every frame.
///
/// The checkbox and select are kept as their **real widget structs** so their own
/// state machines ([`Checkbox::toggle`], [`Select::toggle_open`] /
/// [`Select::select`]) drive the demo; the radio group and input are plain fields
/// (the radio group is composed, and the input's text is mirrored from the
/// runtime's editing session).
#[derive(Debug, Clone)]
pub struct GalleryState {
    /// How many times the enabled button has been clicked.
    pub click_count: u32,
    /// A short human description of the last interaction, shown in the status line.
    pub last_event: String,
    /// The plain checkbox (real widget state machine).
    pub check: Checkbox,
    /// The tri-state checkbox (starts indeterminate).
    pub tri: Checkbox,
    /// The composed radio group's selected index.
    pub radio: usize,
    /// The dropdown (real widget: open/selected state machine).
    pub select: Select,
    /// The editable field's current text, mirrored from the editing session.
    pub input: String,
}

impl Default for GalleryState {
    fn default() -> Self {
        Self {
            click_count: 0,
            last_event: "ready".to_owned(),
            check: Checkbox::new("Enable feature", CMD_CHK),
            tri: Checkbox::indeterminate("Select all", CMD_TRI),
            radio: 0,
            select: Select::new(CMD_SEL_TOGGLE).with_options(SELECT_LABELS),
            input: "Edit me".to_owned(),
        }
    }
}

// ---------------------------------------------------------------------------
// The stylesheet — a neutral dark UA-ish sheet keyed to the controls' semantic
// types/classes, WITH live pseudo-class rules driven by the real pipeline.
// ---------------------------------------------------------------------------

/// The authored stylesheet: a clean dark theme keyed to the widgets' stable CSS
/// types (`button` / `checkbox` / `input`) and the composed elements' classes,
/// with live `:hover` / `:focus` / `:active` / `:checked` / `:disabled` rules.
/// Returned as a `&'static str` so both drivers compile it through the real
/// `obscur-css` frontend.
#[must_use]
pub fn stylesheet() -> &'static str {
    "\
* { box-sizing: border-box; }\n\
#app { display: flex; flex-direction: column; gap: 12px; padding: 20px; \
       background-color: #12151f; width: 100%; height: 100%; }\n\
.title { color: #f5f7fa; font-size: 22px; font-weight: 700; height: 28px; }\n\
.subtitle { color: #8b95ad; font-size: 13px; height: 18px; }\n\
.status { color: #ffd93d; font-size: 13px; font-weight: 600; height: 20px; \
          background-color: #1a1f2e; border-radius: 6px; padding: 4px 8px; }\n\
.section { display: flex; flex-direction: column; gap: 6px; padding: 10px; \
           background-color: #171c29; border-radius: 8px; }\n\
.section-label { color: #aab4c8; font-size: 12px; font-weight: 700; height: 16px; }\n\
.row { display: flex; flex-direction: row; gap: 10px; }\n\
\
/* Base control chrome (shared by every clickable control). */\n\
.ctl { color: #e6ebf5; font-size: 13px; height: 32px; width: 220px; \
       padding: 6px 12px; border-radius: 7px; border: 1px solid #3a4560; \
       background-color: #1f2637; }\n\
\
/* Button. */\n\
button { color: #ffffff; background-color: #3552d9; border: 1px solid #4f6bff; }\n\
button:hover { background-color: #4f6bff; border-color: #7d92ff; }\n\
button:focus { outline: 2px solid #a6b6ff; }\n\
button:active { background-color: #8da3ff; border-color: #c3d0ff; }\n\
button:disabled { background-color: #2a2e3a; color: #6b7280; \
                  border: 1px dashed #4a4f5d; }\n\
\
/* Checkbox. */\n\
checkbox { color: #d7deee; background-color: #1f2637; border: 1px solid #3a4560; }\n\
checkbox:hover { border-color: #6b83ff; background-color: #26304a; }\n\
checkbox:focus { outline: 2px solid #a6b6ff; }\n\
checkbox:checked { color: #ffffff; background-color: #1f6f3d; border-color: #22c55e; }\n\
checkbox:disabled { color: #6b7280; background-color: #23262f; \
                    border: 1px dashed #4a4f5d; }\n\
\
/* Radio group (composed: clickable elements carrying the CHECKED attribute). */\n\
.radio { color: #d7deee; background-color: #1f2637; border: 1px solid #3a4560; }\n\
.radio:hover { border-color: #6b83ff; background-color: #26304a; }\n\
.radio:focus { outline: 2px solid #a6b6ff; }\n\
.radio:checked { color: #ffffff; background-color: #274b8a; border-color: #6ea8ff; }\n\
.radio:disabled { color: #6b7280; background-color: #23262f; \
                  border: 1px dashed #4a4f5d; }\n\
\
/* Select. */\n\
.select-header { color: #e6ebf5; background-color: #1f2637; \
                 border: 1px solid #3a4560; font-weight: 600; }\n\
.select-header:hover { border-color: #6b83ff; background-color: #26304a; }\n\
.select-header:focus { outline: 2px solid #a6b6ff; }\n\
.select-header:disabled { color: #6b7280; background-color: #23262f; \
                          border: 1px dashed #4a4f5d; }\n\
.select-list { display: flex; flex-direction: column; gap: 2px; \
               padding: 4px; width: 220px; background-color: #0f1220; \
               border: 1px solid #4f6bff; border-radius: 7px; }\n\
.option { color: #d7deee; font-size: 13px; height: 28px; padding: 4px 12px; \
          border-radius: 5px; background-color: #171c29; }\n\
.option:hover { background-color: #274b8a; color: #ffffff; }\n\
.option:checked { background-color: #22c55e; color: #10131c; font-weight: 700; }\n\
\
/* Text input. */\n\
input { color: #e6ebf5; background-color: #0f1220; border: 1px solid #3a4560; \
        font-size: 13px; }\n\
input:hover { border-color: #6b83ff; }\n\
input:focus { outline: 2px solid #a6b6ff; border-color: #7d92ff; }\n\
input:disabled { color: #6b7280; background-color: #191c24; \
                 border: 1px dashed #4a4f5d; }\n\
"
}

// ---------------------------------------------------------------------------
// The view builder.
// ---------------------------------------------------------------------------

/// Builds the full gallery view tree from `state`. Pure view/widget/primitive
/// construction — no backend types — so both drivers call it identically.
#[must_use]
pub fn view(state: &GalleryState) -> View {
    let header = section_bare(vec![
        text("title", "obscur.kit — interactive control gallery"),
        text(
            "subtitle",
            "hover · click · focus · type · toggle · open — live pseudo-classes",
        ),
        el(
            "status",
            vec![text_inline(&format!(
                "last event: {}   ·   clicks: {}",
                state.last_event, state.click_count
            ))],
        ),
    ]);

    let buttons = section(
        "Button",
        vec![row(vec![
            widget_ctl("btn", "ctl", Button::new("Primary", CMD_BTN).view()),
            widget_ctl(
                "btn-off",
                "ctl",
                Button::new("Disabled", CMD_BTN_OFF).disabled(true).view(),
            ),
        ])],
    );

    let checkboxes = section(
        "Checkbox (incl. tri-state)",
        vec![
            widget_ctl("chk", "ctl", state.check.view()),
            widget_ctl("chk-tri", "ctl", state.tri.view()),
            widget_ctl(
                "chk-off",
                "ctl",
                Checkbox::new("Disabled", CMD_CHK_OFF).disabled(true).view(),
            ),
        ],
    );

    let radios = section("Radio group (composed)", radio_group(state));

    let selects = section("Select (dropdown)", select_block(state));

    let inputs = section(
        "Text input",
        vec![
            widget_ctl(
                "field",
                "ctl",
                Input::with_value(CMD_FIELD, state.input.clone())
                    .placeholder("Type here…")
                    .view(),
            ),
            widget_ctl(
                "field-off",
                "ctl",
                Input::with_value(CMD_FIELD_OFF, "read only")
                    .disabled(true)
                    .view(),
            ),
        ],
    );

    let app = Container::new()
        .with_children(vec![header, buttons, checkboxes, radios, selects, inputs])
        .into_view_node()
        .with_prop(ViewProperty::new(ID, ViewValue::Text("app".into())))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("app".into())));
    View::single(app)
}

/// Builds the composed radio group: `RADIO_COUNT` enabled clickable rows sharing
/// `state.radio` as the single selected index (the chosen one carries the reserved
/// `CHECKED` attribute so `:checked` fills it), plus one disabled row.
fn radio_group(state: &GalleryState) -> Vec<ViewNode> {
    let mut rows: Vec<ViewNode> = (0..RADIO_COUNT)
        .map(|i| {
            clickable(
                &format!("radio-{i}"),
                "radio ctl",
                radio_command(i),
                state.radio == i, // :checked when selected
                false,
                vec![text_inline(RADIO_LABELS[i])],
            )
        })
        .collect();
    rows.push(clickable(
        "radio-off",
        "radio ctl",
        CMD_RADIO_OFF,
        false,
        true, // :disabled
        vec![text_inline("Unavailable")],
    ));
    rows
}

/// Builds the select block: a clickable header (fires [`CMD_SEL_TOGGLE`], showing
/// the selected label or a prompt), the option list rendered **only while the real
/// [`Select::is_open`] is true** (each option a composed clickable with its own
/// command), and a disabled header variant.
fn select_block(state: &GalleryState) -> Vec<ViewNode> {
    let label = state
        .select
        .selected_label()
        .map_or_else(|| "Choose a fruit…".to_owned(), |s| format!("{s}  ▾"));
    let mut block = vec![clickable(
        "sel-header",
        "select-header ctl",
        CMD_SEL_TOGGLE,
        false,
        false,
        vec![text_inline(&label)],
    )];

    if state.select.is_open() {
        // Open state is STRUCTURAL: the list is in the tree only while the real
        // widget reports open. Each option carries a per-index click command so a
        // windowed hit-test can identify the chosen option (composed routing).
        let options: Vec<ViewNode> = SELECT_LABELS
            .iter()
            .enumerate()
            .map(|(i, lbl)| {
                clickable(
                    &format!("sel-opt-{i}"),
                    "option",
                    select_option_command(i),
                    state.select.selected_index() == Some(i), // :checked on the current pick
                    false,
                    vec![text_inline(lbl)],
                )
            })
            .collect();
        block.push(
            Container::new()
                .with_children(options)
                .into_view_node()
                .with_prop(ViewProperty::new(
                    CLASS,
                    ViewValue::Text("select-list".into()),
                )),
        );
    }

    block.push(clickable(
        "sel-off",
        "select-header ctl",
        CMD_SEL_OFF,
        false,
        true, // :disabled
        vec![text_inline("Disabled select ▾")],
    ));
    block
}

// ---------------------------------------------------------------------------
// Small view helpers.
// ---------------------------------------------------------------------------

/// A `.section` panel with a bold label and `children` stacked below it.
fn section(label: &str, mut children: Vec<ViewNode>) -> ViewNode {
    let mut nodes = vec![text("section-label", label)];
    nodes.append(&mut children);
    el("section", nodes)
}

/// A bare (unlabelled, chrome-less) grouping container carrying no class.
fn section_bare(children: Vec<ViewNode>) -> ViewNode {
    Container::new().with_children(children).into_view_node()
}

/// A `.row` flex container holding `children`.
fn row(children: Vec<ViewNode>) -> ViewNode {
    el("row", children)
}

/// Wraps a widget's rendered root node, tagging it with a unique `id` (so the
/// headless proof can hit-test it by identity) and an additional `class`.
fn widget_ctl(id: &str, class: &str, view: View) -> ViewNode {
    first_node(view)
        .with_prop(ViewProperty::new(ID, ViewValue::Text(id.into())))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}

/// Builds a composed clickable element: a classed [`Container`] carrying a unique
/// `id`, an `EVENT_CLICK` binding to `command`, optional reserved `CHECKED` /
/// `DISABLED` structural-state attributes (driving `:checked` / `:disabled`), and
/// `children`.
fn clickable(
    id: &str,
    class: &str,
    command: InternedSymbol,
    checked: bool,
    disabled: bool,
    children: Vec<ViewNode>,
) -> ViewNode {
    let mut node = Container::new()
        .with_children(children)
        .into_view_node()
        .with_prop(ViewProperty::new(ID, ViewValue::Text(id.into())))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_event(ViewEventBinding::new(EVENT_CLICK, command));
    if checked {
        node = node.with_prop(ViewProperty::new(CHECKED, ViewValue::Bool(true)));
    }
    if disabled {
        node = node.with_prop(ViewProperty::new(DISABLED, ViewValue::Bool(true)));
    }
    node
}

/// A classed [`Container`] element node wrapping `children`.
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

/// A bare (class-less) [`Text`] node used as a control's label child.
fn text_inline(content: &str) -> ViewNode {
    Text::new(content.to_owned()).into_view_node()
}

/// Pulls a widget's first rendered [`ViewNode`] out of the [`View`] it produces.
fn first_node(view: View) -> ViewNode {
    view.into_fragment().nodes().to_vec().remove(0)
}
