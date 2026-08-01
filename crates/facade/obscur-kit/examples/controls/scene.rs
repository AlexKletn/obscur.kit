#![doc = include_str!("scene.doc/_module.md")]
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

#[doc = include_str!("scene.doc/CMD_BTN.md")]
pub const CMD_BTN: InternedSymbol = InternedSymbol::from_raw(0x7000_0001);
#[doc = include_str!("scene.doc/CMD_BTN_OFF.md")]
pub const CMD_BTN_OFF: InternedSymbol = InternedSymbol::from_raw(0x7000_0002);
#[doc = include_str!("scene.doc/CMD_CHK.md")]
pub const CMD_CHK: InternedSymbol = InternedSymbol::from_raw(0x7000_0003);
#[doc = include_str!("scene.doc/CMD_TRI.md")]
pub const CMD_TRI: InternedSymbol = InternedSymbol::from_raw(0x7000_0004);
#[doc = include_str!("scene.doc/CMD_CHK_OFF.md")]
pub const CMD_CHK_OFF: InternedSymbol = InternedSymbol::from_raw(0x7000_0005);
#[doc = include_str!("scene.doc/CMD_SEL_TOGGLE.md")]
pub const CMD_SEL_TOGGLE: InternedSymbol = InternedSymbol::from_raw(0x7000_0006);
#[doc = include_str!("scene.doc/CMD_SEL_OFF.md")]
pub const CMD_SEL_OFF: InternedSymbol = InternedSymbol::from_raw(0x7000_0007);
#[doc = include_str!("scene.doc/CMD_FIELD.md")]
pub const CMD_FIELD: InternedSymbol = InternedSymbol::from_raw(0x7000_0008);
#[doc = include_str!("scene.doc/CMD_FIELD_OFF.md")]
pub const CMD_FIELD_OFF: InternedSymbol = InternedSymbol::from_raw(0x7000_0009);

#[doc = include_str!("scene.doc/RADIO_COUNT.md")]
pub const RADIO_COUNT: usize = 3;
#[doc = include_str!("scene.doc/CMD_RADIO_BASE.md")]
pub const CMD_RADIO_BASE: u32 = 0x7000_0010;
#[doc = include_str!("scene.doc/CMD_RADIO_OFF.md")]
pub const CMD_RADIO_OFF: InternedSymbol = InternedSymbol::from_raw(0x7000_001F);

#[doc = include_str!("scene.doc/SELECT_COUNT.md")]
pub const SELECT_COUNT: usize = 3;
#[doc = include_str!("scene.doc/CMD_SEL_OPT_BASE.md")]
pub const CMD_SEL_OPT_BASE: u32 = 0x7000_0020;

#[doc = include_str!("scene.doc/radio_command.md")]
#[must_use]
pub fn radio_command(index: usize) -> InternedSymbol {
    InternedSymbol::from_raw(CMD_RADIO_BASE + index as u32)
}

#[doc = include_str!("scene.doc/select_option_command.md")]
#[must_use]
pub fn select_option_command(index: usize) -> InternedSymbol {
    InternedSymbol::from_raw(CMD_SEL_OPT_BASE + index as u32)
}

#[doc = include_str!("scene.doc/RADIO_LABELS.md")]
pub const RADIO_LABELS: [&str; RADIO_COUNT] = ["Small", "Medium", "Large"];
#[doc = include_str!("scene.doc/SELECT_LABELS.md")]
pub const SELECT_LABELS: [&str; SELECT_COUNT] = ["Apple", "Banana", "Cherry"];

// ---------------------------------------------------------------------------
// The shared interaction state.
// ---------------------------------------------------------------------------

#[doc = include_str!("scene.doc/GalleryState.md")]
#[derive(Debug, Clone)]
pub struct GalleryState {
    #[doc = include_str!("scene.doc/click_count.md")]
    pub click_count: u32,
    #[doc = include_str!("scene.doc/last_event.md")]
    pub last_event: String,
    #[doc = include_str!("scene.doc/check.md")]
    pub check: Checkbox,
    #[doc = include_str!("scene.doc/tri.md")]
    pub tri: Checkbox,
    #[doc = include_str!("scene.doc/radio.md")]
    pub radio: usize,
    #[doc = include_str!("scene.doc/select.md")]
    pub select: Select,
    #[doc = include_str!("scene.doc/input.md")]
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

#[doc = include_str!("scene.doc/stylesheet.md")]
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

#[doc = include_str!("scene.doc/view.md")]
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

#[doc = include_str!("scene.doc/radio_group.md")]
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

#[doc = include_str!("scene.doc/select_block.md")]
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

#[doc = include_str!("scene.doc/section.md")]
fn section(label: &str, mut children: Vec<ViewNode>) -> ViewNode {
    let mut nodes = vec![text("section-label", label)];
    nodes.append(&mut children);
    el("section", nodes)
}

#[doc = include_str!("scene.doc/section_bare.md")]
fn section_bare(children: Vec<ViewNode>) -> ViewNode {
    Container::new().with_children(children).into_view_node()
}

#[doc = include_str!("scene.doc/row.md")]
fn row(children: Vec<ViewNode>) -> ViewNode {
    el("row", children)
}

#[doc = include_str!("scene.doc/widget_ctl.md")]
fn widget_ctl(id: &str, class: &str, view: View) -> ViewNode {
    first_node(view)
        .with_prop(ViewProperty::new(ID, ViewValue::Text(id.into())))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}

#[doc = include_str!("scene.doc/clickable.md")]
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
