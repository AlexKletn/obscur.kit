#![doc = include_str!("scene.doc/_module.md")]
// All three drivers share this module via `#[path]`; each uses the full surface.
#![allow(dead_code, unreachable_pub)]

use obscur_kit::primitives::{Container, Text};
use obscur_kit::view::view::{CLASS, ID, View, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("scene.doc/stylesheet.md")]
#[must_use]
pub fn stylesheet() -> &'static str {
    "\
* { box-sizing: border-box; }\n\
#app { display: flex; flex-direction: column; gap: 18px; padding: 24px; \
       background-color: #0e1016; }\n\
.header { display: flex; flex-direction: column; gap: 6px; height: 66px; }\n\
.title { color: #f4f6fb; font-size: 30px; font-weight: 700; height: 38px; }\n\
.subtitle { color: #8791a8; font-size: 15px; font-style: italic; height: 20px; }\n\
\
.grid { display: grid; \
        grid-template-columns: 1fr 1fr 1fr 1fr; \
        grid-template-rows: 220px 220px 220px 220px; gap: 16px; height: 928px; }\n\
.card { display: flex; flex-direction: column; gap: 8px; \
        border-radius: 12px; padding: 14px; background-color: #171b26; }\n\
.card-label { color: #e7ecf6; font-size: 14px; font-weight: 700; height: 18px; }\n\
.card-note { color: #98a2ba; font-size: 12px; height: 16px; }\n\
.stage { display: flex; flex-direction: row; align-items: center; \
         gap: 8px; height: 118px; }\n\
\
/* 1 — color spaces (CSS Color 4 parsing → sRGB) */\n\
.swatches { display: flex; flex-direction: row; gap: 10px; }\n\
.sw { display: flex; flex-direction: column; gap: 5px; }\n\
.sw-box { width: 56px; height: 56px; border-radius: 7px; }\n\
.sw-cap { color: #8791a8; font-size: 11px; height: 14px; }\n\
.sw-oklch { background-color: oklch(0.7 0.15 30); }\n\
.sw-lab   { background-color: lab(60% 40 30); }\n\
.sw-hsl   { background-color: hsl(140 70% 45%); }\n\
.sw-oklab { background-color: oklab(0.7 0.1 -0.1); }\n\
\
/* 2 — oklab-interpolated linear gradient (GPU) */\n\
.grad-bar { width: 300px; height: 54px; border-radius: 8px; \
            background-image: linear-gradient(90deg, #ff0000, #00ff00); }\n\
\
/* 3 — linearRGB blur (GPU). The whole panel (a green box on a dark-red field) is\n\
   blurred TOGETHER under one filter layer, so the blur has an internal green↔red\n\
   edge and room inside the clip — the exact bracket the blur proof uses. */\n\
.blur-stage { display: flex; flex-direction: row; align-items: center; \
              justify-content: center; width: 210px; height: 96px; }\n\
.blur-panel { display: flex; flex-direction: row; align-items: center; \
              justify-content: center; width: 200px; height: 92px; \
              border-radius: 8px; background-color: #3a0f12; \
              filter: blur(7px); }\n\
.blur-box { width: 96px; height: 48px; border-radius: 6px; \
            background-color: #00ff00; }\n\
\
/* 4 — border-radius overlap clamp */\n\
.radius-box { width: 200px; height: 40px; border-radius: 30px; \
              background-color: #6ea8ff; }\n\
\
/* 5 — box-shadow spread (a soft halo grown by the spread radius). The box is\n\
   kept square: an element's rounded/overflow clip also clips its OWN outset\n\
   shadow in the current coordinator, so a squared box lets the halo show. */\n\
.shadow-box { width: 128px; height: 56px; \
              background-color: #2b3550; \
              box-shadow: 0 0 12px 8px rgba(125, 185, 255, 0.9); }\n\
\
/* 6 — stacking order (§9.9 positioned above non-positioned) */\n\
.stack-stage { display: block; position: relative; height: 100px; width: 170px; }\n\
.stack-pos { position: relative; z-index: auto; width: 120px; height: 50px; \
             margin-left: 34px; border-radius: 7px; \
             background-color: oklch(0.74 0.15 195); }\n\
.stack-flow { width: 120px; height: 50px; margin-top: -26px; border-radius: 7px; \
              background-color: #e0685f; }\n\
\
/* 7 — flex grow/shrink/basis (§9.7) */\n\
.flex-row { display: flex; flex-direction: row; gap: 6px; \
            width: 290px; height: 52px; }\n\
.fa { flex-grow: 1; flex-shrink: 1; flex-basis: 40px; border-radius: 6px; \
      background-color: #4f6bff; }\n\
.fb { flex-grow: 3; flex-shrink: 1; flex-basis: 40px; border-radius: 6px; \
      background-color: #22d3a6; }\n\
.fc { flex-grow: 0; flex-shrink: 0; flex-basis: 70px; border-radius: 6px; \
      background-color: #ffd93d; }\n\
\
/* 8 — grid two-value tracks (§11 base/growth-limit) */\n\
.grid-demo { display: grid; \
             grid-template-columns: minmax(80px, 1fr) 1fr 1fr; \
             grid-template-rows: 52px; \
             gap: 6px; width: 290px; height: 52px; }\n\
.gc { border-radius: 6px; }\n\
.gc-1 { background-color: #f857a6; }\n\
.gc-2 { background-color: #6ea8ff; }\n\
.gc-3 { background-color: #22d3a6; }\n\
\
/* 9 — margin collapsing (collapses to 30, not 50) */\n\
.collapse-stage { display: block; width: 150px; height: 118px; \
                  background-color: #0e1016; border-radius: 6px; }\n\
.mc-a { width: 150px; height: 34px; margin-bottom: 20px; border-radius: 6px; \
        background-color: #4f6bff; }\n\
.mc-b { width: 150px; height: 34px; margin-top: 30px; border-radius: 6px; \
        background-color: #22d3a6; }\n\
\
/* 10 — letter/word-spacing + text-transform */\n\
.spacing-text { color: #ffd93d; font-size: 17px; font-weight: 600; height: 44px; \
                letter-spacing: 4px; word-spacing: 10px; \
                text-transform: uppercase; }\n\
\
/* 11 — white-space: normal / pre / nowrap */\n\
.ws-cols { display: flex; flex-direction: row; gap: 8px; }\n\
.ws-col { display: flex; flex-direction: column; gap: 4px; width: 88px; }\n\
.ws-cap { color: #8791a8; font-size: 11px; height: 14px; }\n\
.ws-text { font-size: 12px; color: #dbe2f0; background-color: #0e1016; \
           padding: 5px; border-radius: 5px; height: 78px; width: 88px; \
           overflow: hidden; }\n\
.ws-normal { white-space: normal; }\n\
.ws-pre    { white-space: pre; }\n\
.ws-nowrap { white-space: nowrap; }\n\
\
/* 12 — text-overflow: ellipsis */\n\
.ellipsis-box { width: 150px; font-size: 15px; color: #e7ecf6; \
                background-color: #0e1016; padding: 8px; border-radius: 6px; \
                height: 34px; white-space: nowrap; overflow: hidden; \
                text-overflow: ellipsis; }\n\
\
/* 13 — transform interpolation (rotate 45°, full size) */\n\
.xform-stage { display: flex; flex-direction: row; align-items: center; \
               justify-content: center; height: 118px; }\n\
.xform-box { width: 66px; height: 66px; border-radius: 8px; \
             background-color: #f857a6; transform: rotate(45deg); }\n\
\
/* 14 — CPU vs GPU honesty legend */\n\
.legend { display: flex; flex-direction: column; gap: 6px; height: 118px; }\n\
.legend-line { color: #b9c2d6; font-size: 12px; height: 18px; }\n\
.legend-gpu { color: #ffd93d; font-size: 12px; height: 18px; font-weight: 600; }\n\
"
}

#[doc = include_str!("scene.doc/view.md")]
#[must_use]
pub fn view(cpu: bool) -> View {
    let header = el(
        "header",
        vec![
            text("title", "obscur.kit — browser-grade CSS, no browser"),
            text(
                "subtitle",
                "CSS authored as a string, compiled by obscur-css, laid out + painted by the real engine",
            ),
        ],
    );

    let grid = el(
        "grid",
        vec![
            color_spaces_card(),
            gradient_card(cpu),
            blur_card(cpu),
            radius_card(),
            shadow_card(),
            stacking_card(),
            flex_card(),
            grid_card(),
            margin_card(),
            spacing_card(),
            white_space_card(),
            ellipsis_card(),
            transform_card(),
            legend_card(),
        ],
    );

    let app = Container::new()
        .with_children(vec![header, grid])
        .into_view_node()
        .with_prop(ViewProperty::new(ID, ViewValue::Text("app".into())))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("app".into())));
    View::single(app)
}

#[doc = include_str!("scene.doc/color_spaces_card.md")]
fn color_spaces_card() -> ViewNode {
    let swatch = |class: &str, cap: &str| {
        el(
            "sw",
            vec![el(&format!("sw-box {class}"), vec![]), text("sw-cap", cap)],
        )
    };
    card(
        "Color spaces",
        "oklch · lab · hsl · oklab → sRGB",
        vec![el(
            "swatches",
            vec![
                swatch("sw-oklch", "oklch"),
                swatch("sw-lab", "lab"),
                swatch("sw-hsl", "hsl"),
                swatch("sw-oklab", "oklab"),
            ],
        )],
    )
}

#[doc = include_str!("scene.doc/gradient_card.md")]
fn gradient_card(cpu: bool) -> ViewNode {
    let note = if cpu {
        "GPU / Vello — Oklab midpoint (CPU: first stop)"
    } else {
        "interpolated in Oklab (vivid midpoint, not muddy)"
    };
    card(
        "Oklab gradient",
        note,
        vec![el("stage", vec![el("grad-bar", vec![])])],
    )
}

#[doc = include_str!("scene.doc/blur_card.md")]
fn blur_card(cpu: bool) -> ViewNode {
    let note = if cpu {
        "GPU / Vello — convolved in linearRGB (CPU: approx)"
    } else {
        "convolved in linearRGB"
    };
    card(
        "linearRGB blur",
        note,
        vec![el(
            "stage",
            vec![el(
                "blur-stage",
                vec![el("blur-panel", vec![el("blur-box", vec![])])],
            )],
        )],
    )
}

#[doc = include_str!("scene.doc/radius_card.md")]
fn radius_card() -> ViewNode {
    card(
        "border-radius clamp",
        "uniform-scale clamp, no overflow",
        vec![el("stage", vec![el("radius-box", vec![])])],
    )
}

#[doc = include_str!("scene.doc/shadow_card.md")]
fn shadow_card() -> ViewNode {
    card(
        "box-shadow spread",
        "radii grow with spread (concentric)",
        vec![el("stage", vec![el("shadow-box", vec![])])],
    )
}

#[doc = include_str!("scene.doc/stacking_card.md")]
fn stacking_card() -> ViewNode {
    card(
        "Stacking order",
        "§9.9: positioned above non-positioned",
        vec![el(
            "stage",
            vec![el(
                "stack-stage",
                vec![el("stack-pos", vec![]), el("stack-flow", vec![])],
            )],
        )],
    )
}

#[doc = include_str!("scene.doc/flex_card.md")]
fn flex_card() -> ViewNode {
    card(
        "Flex grow/shrink",
        "§9.7 resolve-flexible-lengths",
        vec![el(
            "stage",
            vec![el(
                "flex-row",
                vec![el("fa", vec![]), el("fb", vec![]), el("fc", vec![])],
            )],
        )],
    )
}

#[doc = include_str!("scene.doc/grid_card.md")]
fn grid_card() -> ViewNode {
    card(
        "Grid two-value tracks",
        "§11 base / growth-limit sizing",
        vec![el(
            "stage",
            vec![el(
                "grid-demo",
                vec![
                    el("gc gc-1", vec![]),
                    el("gc gc-2", vec![]),
                    el("gc gc-3", vec![]),
                ],
            )],
        )],
    )
}

#[doc = include_str!("scene.doc/margin_card.md")]
fn margin_card() -> ViewNode {
    card(
        "Margin collapsing",
        "20 + 30 collapse to 30, not 50",
        vec![el(
            "collapse-stage",
            vec![el("mc-a", vec![]), el("mc-b", vec![])],
        )],
    )
}

#[doc = include_str!("scene.doc/spacing_card.md")]
fn spacing_card() -> ViewNode {
    card(
        "Text spacing + case",
        "letter 4px · word 10px · uppercase",
        vec![text("spacing-text", "sharp text stack")],
    )
}

#[doc = include_str!("scene.doc/white_space_card.md")]
fn white_space_card() -> ViewNode {
    // Double spaces + a newline so `normal` collapses, `pre` preserves both, and
    // `nowrap` keeps it on one (overflowing) line.
    const SRC: &str = "two  words\nnew line";
    let col = |cap: &str, class: &str| {
        el(
            "ws-col",
            vec![text("ws-cap", cap), text(&format!("ws-text {class}"), SRC)],
        )
    };
    card(
        "white-space",
        "collapse vs preserve vs one line",
        vec![el(
            "ws-cols",
            vec![
                col("normal", "ws-normal"),
                col("pre", "ws-pre"),
                col("nowrap", "ws-nowrap"),
            ],
        )],
    )
}

#[doc = include_str!("scene.doc/ellipsis_card.md")]
fn ellipsis_card() -> ViewNode {
    card(
        "Ellipsis",
        "text-overflow: ellipsis",
        vec![el(
            "stage",
            vec![text("ellipsis-box", "this line is far too long to fit")],
        )],
    )
}

#[doc = include_str!("scene.doc/transform_card.md")]
fn transform_card() -> ViewNode {
    card(
        "Transform interp",
        "decompose→lerp: rotate stays full-size",
        vec![el("xform-stage", vec![el("xform-box", vec![])])],
    )
}

#[doc = include_str!("scene.doc/legend_card.md")]
fn legend_card() -> ViewNode {
    card(
        "CPU vs GPU",
        "same view, honest per backend",
        vec![el(
            "legend",
            vec![
                text("legend-line", "layout · color · radius · stacking · text"),
                text("legend-line", "render faithfully on CPU + GPU"),
                text("legend-gpu", "gradient + blur: GPU / Vello only"),
            ],
        )],
    )
}

#[doc = include_str!("scene.doc/card.md")]
fn card(label: &str, note: &str, demo: Vec<ViewNode>) -> ViewNode {
    let mut children = vec![text("card-label", label), text("card-note", note)];
    children.extend(demo);
    el("card", children)
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
    el(class, vec![Text::new(content.to_owned()).into_view_node()])
}
