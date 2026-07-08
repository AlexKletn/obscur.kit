//! The **Browser Elements Wireframe Kit** scene: a grayscale, flat "spec sheet"
//! of every native HTML control shown across all of its visual states
//! (default / hover / focus / active / disabled / error / …). This is the
//! toolkit's **base-component / user-agent-stylesheet reference**, authored as one
//! theme-agnostic view built from *semantic* classes plus a single wireframe
//! stylesheet keyed to those classes (so [`stylesheet`] reads like a real UA sheet
//! whose state variants are ordinary CSS classes: `.wf-btn.hover`, `.wf-field.focus`,
//! `.wf-check.checked`, …). Rendered headless to a PNG by the sibling
//! `wireframe_png.rs` example.
//!
//! # Fidelity notes — now authored on REAL primitives
//!
//! An earlier pass of this sheet was a deliberate *gap-finding* exercise: six
//! base-element features the design leans on were only *approximated*. Those gaps
//! have since been closed, so this sheet now renders the AUTHENTIC CSS for each
//! (search "REAL" in the CSS below):
//!
//! - **`outline` + `outline-offset` focus rings** — the focused control carries a
//!   real `outline: 2px dashed #2b2a27; outline-offset: 3px` on the `.wf-focus`
//!   wrapper; the ring paints OUTSIDE the box with a visible offset gap and takes
//!   no layout space (via the coordinator's outline path).
//! - **Dashed borders** (disabled controls, dashed dividers/underlines) — real
//!   `border: 1.5px dashed …`, stroked as actual dashes by both backends.
//! - **CSS-border triangles / arrows** (select `▼` / open `▲`, disclosure `▸` / `▾`,
//!   number-spinner `▲` / `▼`) — the classic `width:0; height:0; border-width:…;
//!   border-color:… transparent …` trick: three transparent sides + one inked side
//!   paint a true triangle (mitered trapezoid quads, transparent sides skipped).
//! - **Graph-paper page grid** — two crossing `repeating-linear-gradient()` layers
//!   in one `background-image` comma list over the page `background-color` (a real
//!   24px grid; GPU-faithful on Vello).
//! - **`repeating-linear-gradient`** hatches — the indeterminate progress hatch,
//!   the meter stripe fills, and the colour-swatch checker are real repeating
//!   gradients (GPU-faithful on Vello; flat first-stop on CPU, by policy).
//! - **Multi-line `<textarea>`** — a single real paragraph string that WRAPS inside
//!   the fixed-size box with a real `line-height` (no manual line splitting).
//! - **Inset `box-shadow` error ring** — a real `box-shadow: 0 0 0 1.5px #2b2a27
//!   inset` painted inside the field box.
//! - **`overflow: hidden`** — the open-select list, the table, the disclosure, the
//!   progress track and the colour swatch clip their children to their rounded
//!   corners.
//!
//! Two things remain genuine gaps (NOT part of this batch, still approximated):
//! **`border-radius: 50%`** circles are given explicit half-px radii, and
//! **`letter-spacing` / `text-transform`** mono captions are authored already-
//! uppercased. The `✓` check, `✕` clear, `⌕` magnifier and `◣` resize grip stay as
//! host-font glyphs (they are not border-triangles).
// One entity per file (xtask forbids logic in mod.rs); both the module and its PNG
// harness could share this, so some items look unused inside a single binary.
#![allow(dead_code, unreachable_pub)]

use obscur_kit::primitives::{Container, Text};
use obscur_kit::view::view::{CLASS, View, ViewNode, ViewProperty, ViewValue};

// ---------------------------------------------------------------------------
// View
// ---------------------------------------------------------------------------

/// Builds the full wireframe sheet: the masthead over the 15 control-group cards
/// and the footer. Pure view/primitive construction — no colours or backend types
/// — so the single [`stylesheet`] drives the whole tree.
#[must_use]
pub fn view() -> View {
    let page = el(
        "wf-page",
        vec![
            masthead(),
            group_button(),
            group_button_primary(),
            group_text_field(),
            group_textarea(),
            group_select(),
            group_checkbox(),
            group_radio(),
            group_range(),
            group_progress(),
            group_number_search(),
            group_file_color(),
            group_link(),
            group_fieldset(),
            group_disclosure(),
            group_scrollbar_table(),
            footer(),
        ],
    );
    let root = el("wf-root", vec![page])
        .with_prop(ViewProperty::new(obscur_kit::view::view::ID, ViewValue::Text("app".into())));
    View::single(root)
}

fn masthead() -> ViewNode {
    el(
        "wf-masthead",
        vec![
            text("wf-mast-eyebrow", "UA STYLESHEET REFERENCE"),
            text("wf-mast-title", "Browser Elements Wireframe Kit"),
            text(
                "wf-mast-sub",
                "Every native HTML control, drawn flat and grayscale across all of its states.",
            ),
        ],
    )
}

fn footer() -> ViewNode {
    el(
        "wf-footer",
        vec![
            text("wf-foot-line", "WIREFRAME UI KIT · 15 GROUPS"),
            text("wf-foot-line", "GRAYSCALE · NO POINTER CURSOR"),
        ],
    )
}

// ---------------------------------------------------------------------------
// 01 · Button
// ---------------------------------------------------------------------------

fn group_button() -> ViewNode {
    let btns = row(vec![
        col("DEFAULT", el("wf-btn", vec![text("wf-btn-t", "Button")])),
        col("HOVER", el("wf-btn hover", vec![text("wf-btn-t", "Button")])),
        // REAL (outline): focus ring is a `.wf-focus` wrapper with a real dashed outline.
        col("FOCUS", focus(el("wf-btn", vec![text("wf-btn-t", "Button")]))),
        col("ACTIVE / PRESSED", el("wf-btn active", vec![text("wf-btn-t", "Button")])),
        // REAL (dashed): the disabled border renders as actual dashes.
        col("DISABLED", el("wf-btn disabled", vec![text("wf-btn-t", "Button")])),
    ]);
    card("01", "Button", "<button>", vec![btns])
}

// ---------------------------------------------------------------------------
// 02 · Button — Primary
// ---------------------------------------------------------------------------

fn group_button_primary() -> ViewNode {
    let btns = row(vec![
        col("DEFAULT", el("wf-btn primary", vec![text("wf-btn-t", "Submit")])),
        col("HOVER", el("wf-btn primary hover", vec![text("wf-btn-t", "Submit")])),
        col("ACTIVE / PRESSED", el("wf-btn primary active", vec![text("wf-btn-t", "Submit")])),
        col("DISABLED", el("wf-btn primary disabled", vec![text("wf-btn-t", "Submit")])),
    ]);
    card("02", "Button — Primary", "<button type=submit>", vec![btns])
}

// ---------------------------------------------------------------------------
// 03 · Text Field
// ---------------------------------------------------------------------------

fn field_text(class: &str, value: &str, placeholder: bool, caret: bool) -> ViewNode {
    let mut inner = vec![text(
        if placeholder { "wf-field-ph" } else { "wf-field-val" },
        value,
    )];
    if caret {
        // SUBSTITUTE (caret): a static 2px×15 bar; no real blinking caret.
        inner.push(el("wf-caret", vec![]));
    }
    el(&format!("wf-field {class}"), vec![el("wf-field-line", inner)])
}

fn group_text_field() -> ViewNode {
    let r1 = row(vec![
        col("DEFAULT / EMPTY", field_text("", "Placeholder", true, false)),
        col("HOVER", field_text("hover", "Placeholder", true, false)),
        col("FOCUS", focus(field_text("focus", "Text", false, true))),
        col("FILLED", field_text("", "Filled value", false, false)),
    ]);
    let r2 = row(vec![
        col("READ-ONLY", field_text("readonly", "Read-only value", false, false)),
        col("DISABLED", field_text("disabled", "Disabled", true, false)),
        // REAL (inset box-shadow error ring): `box-shadow: 0 0 0 1.5px #2b2a27 inset`.
        col(
            "ERROR",
            el(
                "wf-field error",
                vec![el("wf-field-line", vec![text("wf-field-val", "Bad value")])],
            ),
        ),
    ]);
    let err_caption = text("wf-inline-err", "△ This field is required.");
    card("03", "Text Field", "<input type=text>", vec![r1, r2, err_caption])
}

// ---------------------------------------------------------------------------
// 04 · Textarea
// ---------------------------------------------------------------------------

fn group_textarea() -> ViewNode {
    let default_ta = el(
        "wf-textarea",
        vec![
            el("wf-ta-body", vec![text("wf-field-ph", "Placeholder text…")]),
            // SUBSTITUTE (CSS resize triangle): a host-font ◣ grip glyph.
            text("wf-ta-grip", "◣"),
        ],
    );
    // REAL (multi-line): one paragraph string that WRAPS inside the fixed-width box
    // via real line-breaking + `line-height` (no manual line splitting).
    let filled_ta = el(
        "wf-textarea",
        vec![
            el(
                "wf-ta-body",
                vec![text(
                    "wf-ta-text",
                    "The quick brown fox jumps over the lazy dog.",
                )],
            ),
            text("wf-ta-grip", "◣"),
        ],
    );
    let disabled_ta = el(
        "wf-textarea disabled",
        vec![
            el("wf-ta-body", vec![text("wf-field-ph", "Disabled")]),
            text("wf-ta-grip", "◣"),
        ],
    );
    let r = row(vec![
        col("DEFAULT", default_ta),
        col("FOCUS / FILLED", focus(filled_ta)),
        col("DISABLED", disabled_ta),
    ]);
    card("04", "Textarea", "<textarea>", vec![r])
}

// ---------------------------------------------------------------------------
// 05 · Select
// ---------------------------------------------------------------------------

fn select_closed(class: &str, label: &str, up: bool, placeholder: bool) -> ViewNode {
    el(
        &format!("wf-select {class}"),
        vec![
            text(if placeholder { "wf-field-ph" } else { "wf-field-val" }, label),
            // REAL (CSS border-triangle): a 0×0 box with three transparent sides and
            // one inked side paints a true ▼ (closed) / ▲ (open) triangle.
            tri(if up { "wf-tri wf-tri-up" } else { "wf-tri wf-tri-down" }),
        ],
    )
}

fn group_select() -> ViewNode {
    let open = el(
        "wf-select-open",
        vec![
            select_closed("open-head", "Option one", true, false),
            el(
                "wf-select-list",
                vec![
                    el("wf-opt selected", vec![text("wf-opt-t", "Option one")]),
                    el("wf-opt hover", vec![text("wf-opt-t", "Option two")]),
                    el("wf-opt", vec![text("wf-opt-t", "Option three")]),
                    el("wf-opt disabled", vec![text("wf-opt-t", "Option four")]),
                ],
            ),
        ],
    );
    let r = row(vec![
        col("CLOSED", select_closed("", "Choose one", false, true)),
        col("HOVER", select_closed("hover", "Choose one", false, true)),
        col("OPEN", open),
        col("DISABLED", select_closed("disabled", "Choose one", false, true)),
    ]);
    card("05", "Select", "<select>", vec![r])
}

// ---------------------------------------------------------------------------
// 06 · Checkbox
// ---------------------------------------------------------------------------

fn checkbox(class: &str, inner: Vec<ViewNode>) -> ViewNode {
    el(&format!("wf-check {class}"), inner)
}

fn group_checkbox() -> ViewNode {
    let r1 = row(vec![
        col("UNCHECKED", checkbox("", vec![])),
        col("HOVER", checkbox("hover", vec![])),
        col("FOCUS", focus(checkbox("", vec![]))),
        // SUBSTITUTE (CSS check): a host-font ✓ glyph.
        col("CHECKED", checkbox("checked", vec![text("wf-check-mark", "✓")])),
        // SUBSTITUTE (indeterminate dash): a 10×2 filled bar box.
        col("INDETERMINATE", checkbox("checked", vec![el("wf-check-dash", vec![])])),
    ]);
    let r2 = row(vec![
        col("DISABLED", checkbox("disabled", vec![])),
        col("DISABLED · CHECKED", checkbox("disabled checked", vec![text("wf-check-mark faint", "✓")])),
    ]);
    let label_row = el(
        "wf-check-labelrow",
        vec![
            checkbox("checked", vec![text("wf-check-mark", "✓")]),
            text("wf-check-label", "Checkbox with adjacent label"),
        ],
    );
    card("06", "Checkbox", "<input type=checkbox>", vec![r1, r2, label_row])
}

// ---------------------------------------------------------------------------
// 07 · Radio
// ---------------------------------------------------------------------------

fn radio(class: &str, dot: bool) -> ViewNode {
    // SUBSTITUTE (border-radius:50%): explicit half-px radii for the circle + dot.
    let inner = if dot { vec![el("wf-radio-dot", vec![])] } else { vec![] };
    el(&format!("wf-radio {class}"), inner)
}

fn group_radio() -> ViewNode {
    let r = row(vec![
        col("UNSELECTED", radio("", false)),
        col("HOVER", radio("hover", false)),
        col("FOCUS", focus(radio("", false))),
        col("SELECTED", radio("selected", true)),
        col("DISABLED", radio("disabled", false)),
        col("DISABLED · SELECTED", radio("disabled selected", true)),
    ]);
    card("07", "Radio", "<input type=radio>", vec![r])
}

// ---------------------------------------------------------------------------
// 08 · Range Slider
// ---------------------------------------------------------------------------

fn range(class: &str, thumb: &str) -> ViewNode {
    // Track (rail) + fill + absolutely-positioned thumb via left:calc(45% - 8px).
    el(
        &format!("wf-range {class}"),
        vec![
            el("wf-range-rail", vec![]),
            el("wf-range-fill", vec![]),
            el(&format!("wf-range-thumb {thumb}"), vec![]),
        ],
    )
}

fn group_range() -> ViewNode {
    let r = row(vec![
        col("DEFAULT", range("", "")),
        col("HOVER", range("hover", "big")),
        col("FOCUS / DRAGGING", focus(range("focus", "on"))),
        col("DISABLED", range("disabled", "faint")),
    ]);
    card("08", "Range Slider", "<input type=range>", vec![r])
}

// ---------------------------------------------------------------------------
// 09 · Progress & Meter
// ---------------------------------------------------------------------------

fn group_progress() -> ViewNode {
    let determinate = el("wf-progress", vec![el("wf-progress-fill p60", vec![])]);
    // REAL (repeating-linear-gradient hatch): a 45° diagonal hatch fill across the
    // whole track (GPU-faithful on Vello; flat first-stop on CPU by policy).
    let indeterminate = el("wf-progress", vec![el("wf-progress-hatch", vec![])]);
    // REAL (repeating-linear-gradient meter stripes): a 90° vertical-stripe fill.
    let meter_hi = el("wf-progress", vec![el("wf-meter-fill m80", vec![])]);
    let meter_lo = el("wf-progress", vec![el("wf-meter-fill m20", vec![])]);
    let r = row(vec![
        col("PROGRESS · 60%", determinate),
        col("PROGRESS · INDETERMINATE", indeterminate),
        col("METER · OPTIMUM 80%", meter_hi),
        col("METER · LOW 20%", meter_lo),
    ]);
    card("09", "Progress & Meter", "<progress> <meter>", vec![r])
}

// ---------------------------------------------------------------------------
// 10 · Number & Search
// ---------------------------------------------------------------------------

fn group_number_search() -> ViewNode {
    let number = el(
        "wf-number",
        vec![
            el("wf-number-val", vec![text("wf-field-val", "128")]),
            // REAL (CSS border-triangles): the ▲ / ▼ spinner arrows are 0×0
            // border-triangle boxes, the two steppers split by a rule.
            el(
                "wf-number-steppers",
                vec![
                    el("wf-step hover", vec![tri("wf-tri wf-tri-up")]),
                    el("wf-step", vec![tri("wf-tri wf-tri-down")]),
                ],
            ),
        ],
    );
    let search_empty = el(
        "wf-search",
        vec![
            // SUBSTITUTE (magnifier icon): a host-font ⌕ glyph.
            text("wf-search-icon", "⌕"),
            text("wf-field-ph", "Search…"),
        ],
    );
    let search_typed = focus(el(
        "wf-search",
        vec![
            text("wf-search-icon", "⌕"),
            text("wf-field-val", "query text"),
            // SUBSTITUTE (CSS cross): a host-font ✕ glyph inside a circular clear button.
            el("wf-search-clear", vec![text("wf-clear-x", "✕")]),
        ],
    ));
    let r = row(vec![
        col("NUMBER · STEPPER", number),
        col("SEARCH · EMPTY", search_empty),
        col("SEARCH · TYPED + CLEAR", search_typed),
    ]);
    card("10", "Number & Search", "<input type=number|search>", vec![r])
}

// ---------------------------------------------------------------------------
// 11 · File & Color
// ---------------------------------------------------------------------------

fn file_input(class: &str, filename: &str, muted: bool) -> ViewNode {
    el(
        &format!("wf-file {class}"),
        vec![
            el("wf-file-btn", vec![text("wf-file-btn-t", "Choose file")]),
            text(if muted { "wf-field-ph" } else { "wf-field-val" }, filename),
        ],
    )
}

fn group_file_color() -> ViewNode {
    // REAL (repeating-linear-gradient checker): a 45° hatch fill clipped to the
    // rounded swatch (GPU-faithful on Vello; flat first-stop on CPU by policy).
    let color = el("wf-color", vec![el("wf-color-inner", vec![])]);
    let r = row(vec![
        col("FILE · EMPTY", file_input("", "No file selected", true)),
        col("FILE · SELECTED", file_input("", "report-q3.pdf", false)),
        col("FILE · DISABLED", file_input("disabled", "No file selected", true)),
        col("COLOR", color),
    ]);
    card("11", "File & Color", "<input type=file|color>", vec![r])
}

// ---------------------------------------------------------------------------
// 12 · Text Link
// ---------------------------------------------------------------------------

fn link(class: &str, label: &str) -> ViewNode {
    // Underline = border-bottom:1.5px solid on the text node.
    el(&format!("wf-link {class}"), vec![text("wf-link-t", label)])
}

fn group_link() -> ViewNode {
    let r = row(vec![
        col("DEFAULT", link("", "Link text")),
        col("HOVER", link("hover", "Link text")),
        col("FOCUS", focus(link("focus", "Link text"))),
        col("VISITED", link("visited", "Link text")),
        col("DISABLED", link("disabled", "Link text")),
    ]);
    card("12", "Text Link", "<a>", vec![r])
}

// ---------------------------------------------------------------------------
// 13 · Fieldset
// ---------------------------------------------------------------------------

fn fieldset_radio_row(class: &str, dot: bool, label: &str) -> ViewNode {
    el(
        "wf-fs-row",
        vec![radio(class, dot), text("wf-fs-label", label)],
    )
}

fn group_fieldset() -> ViewNode {
    // The legend chip overlaps the top border via position:absolute; top:-9px.
    let fs = el(
        "wf-fieldset",
        vec![
            text("wf-legend", "Legend"),
            el(
                "wf-fs-body",
                vec![
                    fieldset_radio_row("selected", true, "Selected option"),
                    fieldset_radio_row("", false, "Unselected option"),
                    fieldset_radio_row("disabled", false, "Disabled option"),
                ],
            ),
        ],
    );
    card("13", "Fieldset", "<fieldset> <legend>", vec![fs])
}

// ---------------------------------------------------------------------------
// 14 · Disclosure
// ---------------------------------------------------------------------------

fn group_disclosure() -> ViewNode {
    let collapsed = el(
        "wf-details",
        vec![el(
            "wf-summary",
            vec![
                // REAL (CSS border-triangle marker): a ▸ right-pointing triangle.
                el("wf-summary-marker", vec![tri("wf-tri wf-tri-right")]),
                text("wf-summary-t", "Summary heading"),
            ],
        )],
    );
    let expanded = el(
        "wf-details open",
        vec![
            el(
                "wf-summary open",
                vec![
                    // REAL (CSS border-triangle marker): a ▾ down-pointing triangle.
                    el("wf-summary-marker", vec![tri("wf-tri wf-tri-down")]),
                    text("wf-summary-t", "Summary heading"),
                ],
            ),
            el(
                "wf-details-body",
                vec![text(
                    "wf-details-p",
                    "Disclosure content revealed when the summary is expanded.",
                )],
            ),
        ],
    );
    let r = row(vec![
        col("COLLAPSED", collapsed),
        col("EXPANDED", expanded),
    ]);
    card("14", "Disclosure", "<details> <summary>", vec![r])
}

// ---------------------------------------------------------------------------
// 15 · Scrollbar & Table
// ---------------------------------------------------------------------------

fn group_scrollbar_table() -> ViewNode {
    let scrollbar = el(
        "wf-scrollbar",
        vec![el("wf-scroll-thumb", vec![])],
    );
    let table = el(
        "wf-table",
        vec![
            el(
                "wf-thead",
                vec![
                    text("wf-th", "NAME"),
                    text("wf-th", "ROLE"),
                    text("wf-th", "STATUS"),
                ],
            ),
            trow("", "Atlas Inc.", "Owner", "Active"),
            trow("hover", "Beacon LLC", "Editor", "Pending"),
            trow("", "Cobalt Co.", "Viewer", "Draft"),
        ],
    );
    let r = row(vec![
        col("SCROLLBAR", scrollbar),
        col("TABLE", table),
    ]);
    card("15", "Scrollbar & Table", "<table>", vec![r])
}

fn trow(class: &str, name: &str, role: &str, status: &str) -> ViewNode {
    el(
        &format!("wf-trow {class}"),
        vec![
            text("wf-td", name),
            text("wf-td muted", role),
            text("wf-td", status),
        ],
    )
}

// ---------------------------------------------------------------------------
// Structural helpers
// ---------------------------------------------------------------------------

/// A control group card: the mono number + `<h2>` title + muted `<code>` tag header
/// (with a bottom rule) over the group's body rows.
fn card(num: &str, title: &str, tag: &str, mut body: Vec<ViewNode>) -> ViewNode {
    let head = el(
        "wf-head",
        vec![
            text("wf-num", num),
            text("wf-title", title),
            text("wf-tag", tag),
        ],
    );
    let mut kids = vec![head];
    kids.append(&mut body);
    el("wf-card", kids)
}

/// A row of state columns. `flex-wrap` is unsupported, so callers pre-split their
/// columns into rows that fit the card width; each row is a plain flex track.
fn row(cols: Vec<ViewNode>) -> ViewNode {
    el("wf-row", cols)
}

/// One state column: the control above a mono caption.
fn col(caption: &str, control: ViewNode) -> ViewNode {
    el("wf-col", vec![el("wf-col-slot", vec![control]), text("wf-cap", caption)])
}

/// REAL (outline + outline-offset): wraps a control in a `.wf-focus` box carrying a
/// real `outline: 2px dashed #2b2a27; outline-offset: 3px` — the ring paints outside
/// the box with a visible offset gap and takes no layout space.
fn focus(control: ViewNode) -> ViewNode {
    el("wf-focus", vec![control])
}

/// A 0×0 CSS border-triangle box (the arrow/marker classes set the inked side +
/// three transparent sides, so the coordinator's mitered border quads draw a true
/// triangle). Content-less; its size is entirely its border widths.
fn tri(class: &str) -> ViewNode {
    el(class, vec![])
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

// ---------------------------------------------------------------------------
// Stylesheet — the wireframe UA sheet, keyed to the semantic classes above.
// State variants are ordinary classes (`.wf-btn.hover`, `.wf-field.error`, …) so
// this reads like a real user-agent stylesheet with state selectors.
// ---------------------------------------------------------------------------

/// The complete grayscale wireframe stylesheet. Palette/tokens follow the spec:
/// ink `#2b2a27`, muted `#5a584f`, faint `#9b998f`, disabled-ink `#b7b5ad`, page
/// `#eceae5`, surface `#faf9f6`, hover `#e4e2db`, 1.5px borders, 2–4px radii.
#[must_use]
pub const fn stylesheet() -> &'static str {
    STYLESHEET
}

const STYLESHEET: &str = r#"
* { box-sizing: border-box; }

/* REAL (graph-paper page grid): two crossing `repeating-linear-gradient()` layers
   in one `background-image` comma list over the page `background-color` — a real
   24px grid of 1px `#e2e0d9` lines (GPU-faithful on Vello; flat on CPU by policy). */
#app { display: flex; flex-direction: column; background-color: #eceae5; }
.wf-root { display: flex; flex-direction: column; background-color: #eceae5; }
.wf-page { display: flex; flex-direction: column; padding: 40px 40px 56px;
    background-color: #eceae5;
    background-image:
        repeating-linear-gradient(#e2e0d9 0 1px, transparent 1px 24px),
        repeating-linear-gradient(90deg, #e2e0d9 0 1px, transparent 1px 24px); }

/* ---- Masthead ---- */
.wf-masthead { display: flex; flex-direction: column; margin-bottom: 30px; }
.wf-mast-eyebrow { height: 14px; color: #9b998f; font-size: 11px; }
.wf-mast-title { height: 40px; margin-top: 6px; color: #2b2a27; font-size: 32px; font-weight: 700; }
.wf-mast-sub { height: 20px; margin-top: 6px; color: #5a584f; font-size: 14px; }

/* ---- Card shell + header ---- */
.wf-card { display: flex; flex-direction: column; width: 1120px; margin-bottom: 26px;
    padding: 28px 30px; background-color: #faf9f6;
    border: 1.5px solid #2b2a27; border-radius: 4px; }
.wf-head { display: flex; flex-direction: row; align-items: flex-end;
    margin-bottom: 22px; border-bottom: 1.5px solid #2b2a27; padding-bottom: 12px; }
.wf-num { margin-right: 14px; height: 20px; color: #9b998f; font-size: 13px; font-weight: 700; flex-shrink: 0; }
.wf-title { height: 24px; margin-right: 14px; color: #2b2a27; font-size: 19px; font-weight: 700; flex-shrink: 0; }
.wf-tag { height: 20px; color: #9b998f; font-size: 13px; flex-shrink: 0; }

/* ---- State-column grid ---- (fixed control sizes → flex-shrink:0 so columns never
   collapse below their authored width; callers pre-split rows to fit the card.) */
.wf-row { display: flex; flex-direction: row; align-items: flex-start; margin-bottom: 8px; }
/* Columns are authored at a nominal 264px but flex-shrink to share the row evenly
   (an even spec-sheet grid); each control fits within its shared share. */
.wf-col { display: flex; flex-direction: column; width: 264px; margin-right: 8px; margin-bottom: 18px; }
/* No fixed slot height: the caption sits directly under its control. Controls in a
   row share a height, so captions align per row without a detaching min-height. */
.wf-col-slot { display: flex; flex-direction: row; align-items: flex-start; margin-bottom: 12px; }
.wf-cap { height: 14px; color: #9b998f; font-size: 10px; font-weight: 700; }

/* REAL (outline + outline-offset): a real dashed focus ring painted OUTSIDE the
   wrapped control with a 3px offset gap; the outline takes no layout space. */
.wf-focus { display: flex; outline: 2px dashed #2b2a27; outline-offset: 3px; }

/* REAL (CSS border-triangles): a 0×0 box whose size is entirely its border widths;
   three transparent sides + one inked side paint a true triangle. */
/* content-box so the 0×0 is the CONTENT box and the border widths add AROUND it
   (the triangle trick assumes a content-box under the sheet-wide `border-box`).
   The classes are named by their RENDERED direction and authored CSS-correctly:
   a triangle points AWAY from its inked edge, so a down arrow ▼ inks the TOP edge,
   an up arrow ▲ inks the BOTTOM edge, and a right arrow ▶ inks the LEFT edge —
   matching the direct display-list `border_triangle_proof` end-to-end. */
.wf-tri { box-sizing: content-box; width: 0; height: 0; }
.wf-tri-down { border-width: 6px 5px 0 5px; border-style: solid;
    border-color: #2b2a27 transparent transparent transparent; }
.wf-tri-up { border-width: 0 5px 6px 5px; border-style: solid;
    border-color: transparent transparent #2b2a27 transparent; }
.wf-tri-right { border-width: 5px 0 5px 7px; border-style: solid;
    border-color: transparent transparent transparent #2b2a27; }

/* ---- 01/02 · Button ---- */
.wf-btn { display: flex; flex-direction: row; align-items: center; justify-content: center;
    height: 40px; padding: 0 20px; background-color: #faf9f6;
    border: 1.5px solid #2b2a27; border-radius: 2px; }
.wf-btn-t { height: 18px; color: #2b2a27; font-size: 14px; font-weight: 500; text-align: center; }
.wf-btn.hover { background-color: #e4e2db; }
.wf-btn.active { background-color: #2b2a27; }
.wf-btn.active .wf-btn-t { color: #faf9f6; }
/* REAL (dashed): the disabled border strokes as actual dashes. */
.wf-btn.disabled { background-color: #f2f0ea; border: 1.5px dashed #b7b5ad; }
.wf-btn.disabled .wf-btn-t { color: #b7b5ad; }
.wf-btn.primary { background-color: #2b2a27; border-color: #2b2a27; }
.wf-btn.primary .wf-btn-t { color: #faf9f6; }
.wf-btn.primary.hover { background-color: #45433d; border-color: #45433d; }
.wf-btn.primary.active { background-color: #000000; border-color: #000000; }
.wf-btn.primary.disabled { background-color: #dedcd4; border: 1.5px dashed #cbc9c1; }
.wf-btn.primary.disabled .wf-btn-t { color: #cbc9c1; }

/* ---- 03 · Text Field ---- */
.wf-field { display: flex; flex-direction: column; justify-content: center; width: 220px; height: 40px;
    padding: 0 12px; background-color: #faf9f6; border: 1.5px solid #2b2a27; border-radius: 2px; }
.wf-field-line { display: flex; flex-direction: row; align-items: center; height: 18px; }
.wf-field-val { height: 18px; color: #2b2a27; font-size: 14px; }
.wf-field-ph { height: 18px; color: #9b998f; font-size: 14px; }
.wf-field.hover { background-color: #f4f2ec; }
/* SUBSTITUTE (caret): a static bar; no real blinking caret. */
.wf-caret { width: 2px; height: 15px; margin-left: 2px; background-color: #2b2a27; }
.wf-field.readonly { background-color: #f2f0ea; border-color: #b7b5ad; }
.wf-field.disabled { background-color: #f2f0ea; border: 1.5px dashed #b7b5ad; }
.wf-field.disabled .wf-field-ph { color: #b7b5ad; }
/* REAL (inset box-shadow error ring): a real inner ring painted inside the box. */
.wf-field.error { border: 1.5px solid #2b2a27; box-shadow: 0 0 0 1.5px #2b2a27 inset; }
.wf-inline-err { height: 16px; margin-top: 2px; color: #2b2a27; font-size: 12px; }

/* ---- 04 · Textarea ---- */
.wf-textarea { position: relative; display: flex; flex-direction: column; width: 240px; height: 96px;
    overflow: hidden; padding: 10px 12px; background-color: #faf9f6;
    border: 1.5px solid #2b2a27; border-radius: 2px; }
.wf-ta-body { display: flex; flex-direction: column; width: 100%; }
/* REAL (multi-line): one paragraph that WRAPS inside the box with a real line-height. */
.wf-ta-text { width: 100%; color: #2b2a27; font-size: 14px; line-height: 20px; }
.wf-textarea.disabled { background-color: #f2f0ea; border: 1.5px dashed #b7b5ad; }
/* SUBSTITUTE (CSS resize triangle): a ◣ grip glyph pinned bottom-right. */
.wf-ta-grip { position: absolute; right: 3px; bottom: 1px; width: 14px; height: 14px;
    color: #9b998f; font-size: 12px; text-align: right; }

/* ---- 05 · Select ---- */
.wf-select { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    width: 200px; height: 40px; padding: 0 12px; background-color: #faf9f6;
    border: 1.5px solid #2b2a27; border-radius: 2px; }
.wf-select.hover { background-color: #e4e2db; }
.wf-select.disabled { background-color: #f2f0ea; border: 1.5px dashed #b7b5ad; }
.wf-select-open { display: flex; flex-direction: column; width: 200px; }
.wf-select .open-head, .open-head { border-radius: 2px 2px 0 0; }
/* REAL (overflow: hidden): the dropped list clips its options to the rounded
   bottom corners (the selected/hover fills stop at the radius). */
.wf-select-list { display: flex; flex-direction: column; width: 200px; overflow: hidden;
    border: 1.5px solid #2b2a27; border-top: 0; border-radius: 0 0 2px 2px; background-color: #faf9f6; }
.wf-opt { display: flex; flex-direction: row; align-items: center; height: 30px; padding: 0 12px; }
.wf-opt-t { height: 18px; color: #2b2a27; font-size: 13px; }
.wf-opt.selected { background-color: #2b2a27; }
.wf-opt.selected .wf-opt-t { color: #faf9f6; }
.wf-opt.hover { background-color: #e4e2db; }
.wf-opt.disabled .wf-opt-t { color: #b7b5ad; }

/* ---- 06 · Checkbox ---- */
.wf-check { display: flex; flex-direction: row; align-items: center; justify-content: center;
    width: 20px; height: 20px; background-color: #faf9f6; border: 1.5px solid #2b2a27; border-radius: 2px; }
.wf-check.hover { background-color: #e4e2db; }
.wf-check.checked { background-color: #2b2a27; }
/* SUBSTITUTE (CSS check): a ✓ glyph. */
.wf-check-mark { width: 14px; height: 14px; color: #faf9f6; font-size: 12px; text-align: center; }
.wf-check-mark.faint { color: #b7b5ad; }
/* SUBSTITUTE (indeterminate dash): a 10×2 filled bar. */
.wf-check-dash { width: 10px; height: 2px; background-color: #faf9f6; }
.wf-check.disabled { background-color: #f2f0ea; border: 1.5px dashed #b7b5ad; }
.wf-check.disabled.checked { background-color: #f2f0ea; }
.wf-check-labelrow { display: flex; flex-direction: row; align-items: center; height: 26px; margin-top: 4px; }
.wf-check-label { height: 18px; margin-left: 10px; color: #2b2a27; font-size: 14px; }

/* ---- 07 · Radio ---- SUBSTITUTE (border-radius:50%): explicit half-px radii. */
.wf-radio { display: flex; flex-direction: row; align-items: center; justify-content: center;
    width: 20px; height: 20px; background-color: #faf9f6; border: 1.5px solid #2b2a27; border-radius: 10px; }
.wf-radio.hover { background-color: #e4e2db; }
.wf-radio-dot { width: 10px; height: 10px; border-radius: 5px; background-color: #2b2a27; }
.wf-radio.disabled { background-color: #f2f0ea; border: 1.5px dashed #b7b5ad; }
.wf-radio.disabled .wf-radio-dot { background-color: #b7b5ad; }

/* ---- 08 · Range Slider ---- (position:absolute + calc thumb) */
.wf-range { position: relative; width: 220px; height: 20px; }
.wf-range-rail { position: absolute; left: 0; right: 0; top: 9px; height: 2px; background-color: #b7b5ad; }
.wf-range-fill { position: absolute; left: 0; top: 9px; width: 45%; height: 2px; background-color: #2b2a27; }
.wf-range-thumb { position: absolute; left: calc(45% - 8px); top: 2px; width: 16px; height: 16px;
    border-radius: 8px; background-color: #faf9f6; border: 1.5px solid #2b2a27; }
.wf-range-thumb.big { left: calc(45% - 10px); top: 0; width: 20px; height: 20px; border-radius: 10px;
    background-color: #e4e2db; }
.wf-range-thumb.on { background-color: #2b2a27; }
.wf-range-thumb.faint { border-color: #b7b5ad; }
.wf-range.disabled .wf-range-fill { background-color: #b7b5ad; }

/* ---- 09 · Progress & Meter ---- (overflow:hidden clips fills to the radius) */
.wf-progress { display: flex; flex-direction: row; width: 220px; height: 14px; overflow: hidden;
    background-color: #faf9f6; border: 1.5px solid #2b2a27; border-radius: 2px; }
.wf-progress-fill { height: 100%; background-color: #2b2a27; }
.wf-progress-fill.p60 { width: 60%; }
/* REAL (repeating-linear-gradient hatch): a 45° diagonal hatch across the track. */
.wf-progress-hatch { width: 100%; height: 100%;
    background-image: repeating-linear-gradient(45deg, #2b2a27 0 8px, #faf9f6 8px 16px); }
/* REAL (repeating-linear-gradient meter stripes): a 90° vertical-stripe fill. */
.wf-meter-fill { height: 100%; }
.wf-meter-fill.m80 { width: 80%;
    background-image: repeating-linear-gradient(90deg, #5a584f 0 6px, #faf9f6 6px 12px); }
.wf-meter-fill.m20 { width: 20%;
    background-image: repeating-linear-gradient(90deg, #9b998f 0 6px, #faf9f6 6px 12px); }

/* ---- 10 · Number & Search ---- */
.wf-number { display: flex; flex-direction: row; align-items: stretch; width: 160px; height: 40px;
    overflow: hidden; background-color: #faf9f6; border: 1.5px solid #2b2a27; border-radius: 2px; }
.wf-number-val { display: flex; flex-direction: row; align-items: center; padding: 0 12px; width: 120px; }
.wf-number-steppers { display: flex; flex-direction: column; width: 38px; border-left: 1.5px solid #2b2a27; }
.wf-step { display: flex; flex-direction: row; align-items: center; justify-content: center; height: 100%; }
.wf-step.hover { background-color: #e4e2db; }
.wf-search { display: flex; flex-direction: row; align-items: center; width: 220px; height: 40px;
    padding: 0 10px; background-color: #faf9f6; border: 1.5px solid #2b2a27; border-radius: 2px; }
/* SUBSTITUTE (magnifier icon): a ⌕ glyph. */
.wf-search-icon { width: 20px; height: 18px; color: #5a584f; font-size: 15px; }
.wf-search-clear { display: flex; flex-direction: row; align-items: center; justify-content: center;
    width: 18px; height: 18px; margin-left: 8px; border-radius: 9px; background-color: #e4e2db; }
/* SUBSTITUTE (CSS cross): a ✕ glyph. */
.wf-clear-x { height: 12px; color: #2b2a27; font-size: 10px; text-align: center; }

/* ---- 11 · File & Color ---- */
.wf-file { display: flex; flex-direction: row; align-items: center; width: 240px; height: 40px;
    background-color: #faf9f6; border: 1.5px solid #2b2a27; border-radius: 2px; }
.wf-file-btn { display: flex; flex-direction: row; align-items: center; justify-content: center;
    height: 100%; padding: 0 14px; background-color: #e4e2db; border-right: 1.5px solid #2b2a27; }
.wf-file-btn-t { height: 16px; color: #2b2a27; font-size: 13px; }
.wf-file .wf-field-ph, .wf-file .wf-field-val { margin-left: 12px; }
.wf-file.disabled { background-color: #f2f0ea; border: 1.5px dashed #b7b5ad; }
.wf-file.disabled .wf-file-btn { background-color: #f2f0ea; border-right: 1.5px dashed #b7b5ad; }
.wf-color { width: 52px; height: 32px; padding: 3px; overflow: hidden; background-color: #faf9f6;
    border: 1.5px solid #2b2a27; border-radius: 2px; }
/* REAL (repeating-linear-gradient checker): a 45° hatch fill (GPU-faithful). */
.wf-color-inner { width: 100%; height: 100%;
    background-image: repeating-linear-gradient(45deg, #9b998f 0 6px, #d6d4cc 6px 12px); }

/* ---- 12 · Text Link ---- (underline = border-bottom) */
.wf-link { display: flex; flex-direction: row; align-items: center; height: 22px; }
.wf-link-t { height: 20px; color: #2b2a27; font-size: 15px; border-bottom: 1.5px solid #2b2a27; }
.wf-link.hover .wf-link-t { color: #000000; font-weight: 700; border-bottom-color: #000000; }
/* REAL (dashed underline): the focus underline renders as actual dashes. */
.wf-link.focus .wf-link-t { border-bottom: 1.5px dashed #2b2a27; }
.wf-link.visited .wf-link-t { color: #7a786f; border-bottom-color: #7a786f; }
.wf-link.disabled .wf-link-t { color: #b7b5ad; border-bottom: 0; }

/* ---- 13 · Fieldset ---- (legend chip overlaps top border, position:absolute) */
.wf-fieldset { position: relative; width: 300px; margin-top: 10px; padding: 20px 18px 8px;
    background-color: #faf9f6; border: 1.5px solid #2b2a27; border-radius: 3px; }
.wf-legend { position: absolute; left: 14px; top: -9px; height: 18px; padding: 0 6px;
    background-color: #faf9f6; color: #2b2a27; font-size: 13px; font-weight: 700; }
.wf-fs-body { display: flex; flex-direction: column; }
.wf-fs-row { display: flex; flex-direction: row; align-items: center; height: 30px; margin-bottom: 8px; }
.wf-fs-label { height: 18px; margin-left: 10px; color: #2b2a27; font-size: 14px; }
.wf-fs-row .wf-radio.disabled ~ .wf-fs-label { color: #b7b5ad; }

/* ---- 14 · Disclosure ---- (overflow:hidden clips the open row to the radius) */
.wf-details { display: flex; flex-direction: column; width: 300px; overflow: hidden;
    border: 1.5px solid #2b2a27; border-radius: 3px; background-color: #faf9f6; }
.wf-summary { display: flex; flex-direction: row; align-items: center; height: 40px; padding: 0 14px; }
.wf-summary.open { background-color: #e4e2db; border-bottom: 1.5px solid #2b2a27; }
/* REAL (CSS border-triangle marker): a box centering the ▸ / ▾ triangle. */
.wf-summary-marker { display: flex; flex-direction: row; align-items: center; justify-content: center;
    width: 18px; height: 18px; margin-right: 4px; }
.wf-summary-t { height: 18px; color: #2b2a27; font-size: 14px; font-weight: 500; }
.wf-details-body { display: flex; flex-direction: column; padding: 12px 14px; }
.wf-details-p { height: 18px; color: #5a584f; font-size: 13px; }

/* ---- 15 · Scrollbar & Table ---- */
.wf-scrollbar { display: flex; flex-direction: column; width: 20px; height: 150px;
    background-color: #faf9f6; border: 1.5px solid #2b2a27; border-radius: 2px; padding: 2px; }
.wf-scroll-thumb { width: 100%; height: 44%; background-color: #e4e2db; border-radius: 2px; }
/* REAL (overflow: hidden): the header/rows clip to the table's rounded corners. */
.wf-table { display: flex; flex-direction: column; width: 250px; overflow: hidden;
    border: 1.5px solid #2b2a27; border-radius: 2px; }
.wf-thead { display: grid; grid-template-columns: 1.4fr 1fr 0.9fr; align-items: center;
    height: 34px; padding: 0 12px; background-color: #2b2a27; }
.wf-th { height: 16px; color: #faf9f6; font-size: 11px; font-weight: 700; }
.wf-trow { display: grid; grid-template-columns: 1.4fr 1fr 0.9fr; align-items: center;
    height: 34px; padding: 0 12px; border-top: 1px solid #d6d4cc; }
.wf-trow.hover { background-color: #e4e2db; }
.wf-td { height: 18px; color: #2b2a27; font-size: 13px; }
.wf-td.muted { color: #5a584f; }

/* ---- Footer ---- */
.wf-footer { display: flex; flex-direction: column; margin-top: 8px; }
.wf-foot-line { height: 16px; color: #9b998f; font-size: 11px; }
"#;
