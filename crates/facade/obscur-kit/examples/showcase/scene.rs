//! The **shared capability-showcase** scene, authored once and rendered by both
//! the headless CPU→PNG example (`showcase_png.rs`) and the windowed Vello
//! example. It is deliberately **backend-pure**: it names only the toolkit's
//! view / widget / primitive / CSS authoring surfaces (`obscur_kit::*`) and never
//! a renderer, window, or GPU type, so the exact same view + stylesheet feed a
//! software rasterizer and a GPU surface unchanged.
//!
//! # What it exercises
//!
//! A titled "gallery" laid out with a **grid** of cards over a **flex** toolbar
//! row, each card demonstrating one shipped capability:
//!
//! - layout: a `display: grid` gallery (multiple tracks) + a `display: flex` row;
//! - backgrounds: `background-color`, `linear-gradient`, `radial-gradient`,
//!   `conic-gradient`;
//! - borders: `border` + `border-radius`, and a thick colored border;
//! - effects: `box-shadow`, `filter: drop-shadow(...)`, group `opacity: 0.5`,
//!   `transform: rotate()/scale()`;
//! - images: a `background-image: url(...)` card and an `<img>` [`Image`]
//!   primitive with `object-fit: contain`;
//! - text: headings + paragraphs at several sizes / weights (a mixed-script
//!   sample too);
//! - widgets: a [`Button`], a [`Checkbox`], and an [`Input`] carrying a value;
//! - a state-driven counter value rendered as text.
//!
//! The gradient / shadow / blur effects are GPU-class: on the CPU backend they
//! degrade to a documented approximation (a gradient paints its first stop; a
//! shadow paints a flat offset rect) per the project's CPU/GPU effect-split
//! policy — the layout, text, images, borders, radius, opacity, and transform
//! geometry all render faithfully. The windowed Vello example shows the effects
//! at full fidelity from the very same module.
// Both examples share this module via `#[path]`; each uses a subset (so some
// items look dead/unreachable-pub inside a single example binary, but are part of
// the shared surface the sibling example also consumes).
#![allow(dead_code, unreachable_pub)]

use obscur_kit::identity::InternedSymbol;
use obscur_kit::primitives::{Container, Image, Text};
use obscur_kit::view::view::{CLASS, ID, View, ViewNode, ViewProperty, ViewValue};
use obscur_kit::widgets::button::Button;
use obscur_kit::widgets::checkbox::Checkbox;
use obscur_kit::widgets::input::Input;

/// The file name the image cards reference. Both `background-image: url(...)` and
/// the `<img>` [`Image`] primitive resolve this against the resource loader's
/// sandbox root, so an example writes [`image_asset_bytes`] to
/// `<sandbox>/gallery.png` before rendering.
pub const IMAGE_ASSET_NAME: &str = "gallery.png";

/// The bytes of the small colorful RGBA PNG the image cards display, embedded so
/// the demo needs no external asset. An example writes these to
/// `<sandbox>/gallery.png` and points the resource loader at that directory.
#[must_use]
pub fn image_asset_bytes() -> &'static [u8] {
    include_bytes!("gallery.png")
}

/// Commands the interactive widgets fire (interned handles); the windowed example
/// binds real command handlers to these, the headless PNG example just renders
/// them.
const CMD_BUTTON: InternedSymbol = InternedSymbol::from_raw(0x5001);
const CMD_CHECKBOX: InternedSymbol = InternedSymbol::from_raw(0x5002);
const CMD_INPUT: InternedSymbol = InternedSymbol::from_raw(0x5003);

/// The small, serializable state the showcase view is parameterized by, so
/// sub-step 2 can drive it interactively. Sub-step 1 renders [`Default`].
#[derive(Debug, Clone)]
pub struct ShowcaseState {
    /// The counter value rendered as text (bumped by the button in the windowed
    /// example).
    pub counter: i32,
    /// The text seeded into the [`Input`] widget.
    pub input_text: String,
    /// The [`Checkbox`] checked state.
    pub checked: bool,
}

impl Default for ShowcaseState {
    fn default() -> Self {
        Self {
            counter: 3,
            input_text: "obscur".to_owned(),
            checked: true,
        }
    }
}

/// The authored stylesheet exercising the selector engine + cascade: id, class,
/// and descendant selectors driving every capability the scene shows. Returned as
/// a `&'static str` so both examples compile it through the real `obscur-css`
/// frontend.
#[must_use]
pub fn stylesheet() -> &'static str {
    "\
* { box-sizing: border-box; }\n\
#app { display: flex; flex-direction: column; gap: 14px; padding: 20px; \
       min-height: 100vh; background-color: #12151f; }\n\
.header { display: flex; flex-direction: column; gap: 4px; height: 62px; }\n\
.title { color: #f5f7fa; font-size: 30px; font-weight: 700; height: 36px; }\n\
.subtitle { color: #8b95ad; font-size: 15px; font-style: italic; height: 20px; }\n\
\
.gallery { display: grid; grid-template-columns: 1fr 1fr 1fr 1fr; \
           grid-template-rows: 158px 158px 158px; gap: 12px; \
           height: 498px; }\n\
.card { border-radius: 10px; padding: 12px; background-color: #1c2130; }\n\
.card-label { color: #e6ebf5; font-size: 14px; font-weight: 600; height: 18px; }\n\
.card-note { color: #aab4c8; font-size: 12px; height: 15px; }\n\
\
.solid { background-color: #3552d9; }\n\
.linear { background-image: linear-gradient(90deg, #ff6b6b, #ffd93d); }\n\
.radial { background-image: radial-gradient(circle, #22d3a6, #0f5f8a); }\n\
.conic { background-image: conic-gradient(from 0deg, #f857a6, #ff5858, #f857a6); }\n\
.bordered { border: 5px solid #ffd93d; border-radius: 14px; background-color: #23283a; }\n\
.shadowed { background-color: #2a3550; box-shadow: 6px 8px 18px 0 rgba(0,0,0,0.55); }\n\
.dropshadow { background-color: #4b3a6b; filter: drop-shadow(4px 6px 8px rgba(0,0,0,0.6)); }\n\
.faded { background-color: #e63946; opacity: 0.5; }\n\
.rotated { background-color: #2a9d8f; transform: rotate(8deg) scale(0.92); }\n\
.bgimage { background-image: url(\"gallery.png\"); background-size: contain; \
           background-repeat: no-repeat; background-position: center; \
           background-color: #0f1220; }\n\
.imgcard { display: flex; flex-direction: column; gap: 6px; background-color: #1a1f2e; }\n\
.thumb { width: 84px; height: 84px; object-fit: contain; }\n\
.widgets { display: flex; flex-direction: column; gap: 6px; background-color: #202638; }\n\
\
.toolbar { display: flex; flex-direction: row; gap: 10px; align-items: center; \
           height: 52px; padding: 8px; border-radius: 10px; \
           background-color: #1a1f2e; }\n\
.chip { border-radius: 8px; padding: 6px; width: 96px; height: 28px; }\n\
.chip-text { color: #10131c; font-size: 13px; font-weight: 700; height: 16px; }\n\
.chip-a { background-color: #ffd93d; }\n\
.chip-b { background-color: #22d3a6; }\n\
.chip-c { background-color: #6ea8ff; }\n\
.counter { color: #ffd93d; font-size: 18px; font-weight: 700; width: 160px; height: 24px; }\n\
\
.btn { background-color: #3552d9; color: #ffffff; border-radius: 8px; \
       border: 1px solid #4f6bff; padding: 8px; height: 30px; \
       font-size: 13px; font-weight: 600; }\n\
.btn:hover { background-color: #4f6bff; border-color: #7d92ff; }\n\
.btn:active { background-color: #6b83ff; border-color: #a6b6ff; }\n\
.btn:focus { background-color: #4f6bff; border-color: #a6b6ff; }\n\
.check { color: #d7deee; font-size: 13px; height: 18px; }\n\
.field { background-color: #0f1220; color: #e6ebf5; border: 1px solid #3a4560; \
         border-radius: 6px; padding: 5px; height: 24px; font-size: 13px; }\n\
"
}

/// Builds the full showcase view tree from `state`. Pure view/widget/primitive
/// construction — no backend types — so both examples call it identically.
#[must_use]
pub fn view(state: &ShowcaseState) -> View {
    let header = el(
        "header",
        vec![
            text("title", "obscur.kit — capability showcase"),
            text("subtitle", "grid + flex · gradients · shadows · images · text · widgets"),
        ],
    );

    let gallery = el(
        "gallery",
        vec![
            card("solid", "Solid fill", "background-color"),
            card("linear", "Linear gradient", "first-stop on CPU"),
            card("radial", "Radial gradient", "first-stop on CPU"),
            card("conic", "Conic gradient", "first-stop on CPU"),
            card("bordered", "Border + radius", "5px solid, r=14"),
            card("shadowed", "Box shadow", "flat approx on CPU"),
            card("dropshadow", "Drop shadow", "filter: drop-shadow"),
            card("faded", "Opacity 0.5", "group layer"),
            card("rotated", "Transform", "rotate + scale"),
            bg_image_card(),
            img_card(),
            widgets_card(state),
        ],
    );

    let toolbar = el(
        "toolbar",
        vec![
            chip("chip-a", "Layout"),
            chip("chip-b", "Paint"),
            chip("chip-c", "Text"),
            text("counter", &format!("counter = {}", state.counter)),
        ],
    );

    let app = Container::new()
        .with_children(vec![header, gallery, toolbar])
        .into_view_node()
        .with_prop(ViewProperty::new(ID, ViewValue::Text("app".into())))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("app".into())));
    View::single(app)
}

/// A toolbar chip: a colored rounded container with a dark centered label — a
/// flex-row item whose `background-color` paints (unlike a bare text node).
fn chip(color: &str, label: &str) -> ViewNode {
    el(&format!("chip {color}"), vec![text("chip-text", label)])
}

/// A gallery card: a classed container with a bold label and a small note.
fn card(class: &str, label: &str, note: &str) -> ViewNode {
    el(
        &format!("card {class}"),
        vec![text("card-label", label), text("card-note", note)],
    )
}

/// The `background-image: url(...)` card (no extra children — the CSS paints it).
fn bg_image_card() -> ViewNode {
    el("card bgimage", vec![text("card-label", "background-image")])
}

/// The `<img>` primitive card: a real [`Image`] element sized by `object-fit`.
fn img_card() -> ViewNode {
    let thumb = Image::new(IMAGE_ASSET_NAME)
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("thumb".into())));
    el("card imgcard", vec![text("card-label", "<img> object-fit"), thumb])
}

/// The widgets card: a real [`Button`], [`Checkbox`], and [`Input`] laid out in
/// a column, each pulled to its element node so it participates in the tree.
fn widgets_card(state: &ShowcaseState) -> ViewNode {
    let button = first_node(Button::new("Click me", CMD_BUTTON).view())
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("btn".into())));
    let check = first_node(Checkbox::new("Enabled", CMD_CHECKBOX).checked(state.checked).view())
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("check".into())));
    let field = first_node(
        Input::with_value(CMD_INPUT, state.input_text.clone())
            .placeholder("type…")
            .view(),
    )
    .with_prop(ViewProperty::new(CLASS, ViewValue::Text("field".into())));
    el("card widgets", vec![text("card-label", "Widgets"), button, check, field])
}

/// Wraps `children` in a classed [`Container`] element node.
fn el(class: &str, children: Vec<ViewNode>) -> ViewNode {
    Container::new()
        .with_children(children)
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}

/// A classed [`Text`] node (the class carries color / size / weight so the run's
/// brush + font style are authored).
fn text(class: &str, content: &str) -> ViewNode {
    Text::new(content.to_owned())
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}

/// Pulls a widget's first rendered [`ViewNode`] out of the [`View`] it produces,
/// so it can be nested as a child of another element.
fn first_node(view: View) -> ViewNode {
    view.into_fragment().nodes().to_vec().remove(0)
}
