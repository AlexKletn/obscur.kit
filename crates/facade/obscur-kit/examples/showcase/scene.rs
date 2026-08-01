#![doc = include_str!("scene.doc/_module.md")]
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

#[doc = include_str!("scene.doc/IMAGE_ASSET_NAME.md")]
pub const IMAGE_ASSET_NAME: &str = "gallery.png";

#[doc = include_str!("scene.doc/image_asset_bytes.md")]
#[must_use]
pub fn image_asset_bytes() -> &'static [u8] {
    include_bytes!("gallery.png")
}

#[doc = include_str!("scene.doc/CMD_BUTTON.md")]
const CMD_BUTTON: InternedSymbol = InternedSymbol::from_raw(0x5001);
const CMD_CHECKBOX: InternedSymbol = InternedSymbol::from_raw(0x5002);
const CMD_INPUT: InternedSymbol = InternedSymbol::from_raw(0x5003);

#[doc = include_str!("scene.doc/ShowcaseState.md")]
#[derive(Debug, Clone)]
pub struct ShowcaseState {
    #[doc = include_str!("scene.doc/counter.md")]
    pub counter: i32,
    #[doc = include_str!("scene.doc/input_text.md")]
    pub input_text: String,
    #[doc = include_str!("scene.doc/checked.md")]
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

#[doc = include_str!("scene.doc/stylesheet.md")]
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

#[doc = include_str!("scene.doc/view.md")]
#[must_use]
pub fn view(state: &ShowcaseState) -> View {
    let header = el(
        "header",
        vec![
            text("title", "obscur.kit — capability showcase"),
            text(
                "subtitle",
                "grid + flex · gradients · shadows · images · text · widgets",
            ),
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

#[doc = include_str!("scene.doc/chip.md")]
fn chip(color: &str, label: &str) -> ViewNode {
    el(&format!("chip {color}"), vec![text("chip-text", label)])
}

#[doc = include_str!("scene.doc/card.md")]
fn card(class: &str, label: &str, note: &str) -> ViewNode {
    el(
        &format!("card {class}"),
        vec![text("card-label", label), text("card-note", note)],
    )
}

#[doc = include_str!("scene.doc/bg_image_card.md")]
fn bg_image_card() -> ViewNode {
    el("card bgimage", vec![text("card-label", "background-image")])
}

#[doc = include_str!("scene.doc/img_card.md")]
fn img_card() -> ViewNode {
    let thumb = Image::new(IMAGE_ASSET_NAME)
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("thumb".into())));
    el(
        "card imgcard",
        vec![text("card-label", "<img> object-fit"), thumb],
    )
}

#[doc = include_str!("scene.doc/widgets_card.md")]
fn widgets_card(state: &ShowcaseState) -> ViewNode {
    let button = first_node(Button::new("Click me", CMD_BUTTON).view())
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("btn".into())));
    let check = first_node(
        Checkbox::new("Enabled", CMD_CHECKBOX)
            .checked(state.checked)
            .view(),
    )
    .with_prop(ViewProperty::new(CLASS, ViewValue::Text("check".into())));
    let field = first_node(
        Input::with_value(CMD_INPUT, state.input_text.clone())
            .placeholder("type…")
            .view(),
    )
    .with_prop(ViewProperty::new(CLASS, ViewValue::Text("field".into())));
    el(
        "card widgets",
        vec![text("card-label", "Widgets"), button, check, field],
    )
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

#[doc = include_str!("scene.doc/first_node.md")]
fn first_node(view: View) -> ViewNode {
    view.into_fragment().nodes().to_vec().remove(0)
}
