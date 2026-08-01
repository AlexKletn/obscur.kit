#![doc = include_str!("spec_ch4_visual.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch4_support/mod.rs"]
mod support;

use obscur_kit::geometry::{Point, Rect, Size};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::style::property::PropertyId;
use support::{approx, at, clip_bounds, drive, el, geom, mapped_bounds, transforms, view};

const VW: f32 = 1000.0;

#[doc = include_str!("spec_ch4_visual.doc/shadow_radii.md")]
fn shadow_radii(rt: &UiRuntime) -> obscur_kit::geometry::CornerRadii {
    rt.snapshot()
        .display_list()
        .commands()
        .iter()
        .find_map(|c| match c {
            PaintCommand::DrawShadow { radii, .. } => Some(*radii),
            _ => None,
        })
        .expect("the box emitted a shadow carrying its used corner radii")
}

#[doc = include_str!("spec_ch4_visual.doc/transform_origin_defaults_to_the_box_centre.md")]
#[test]
fn transform_origin_defaults_to_the_box_centre() {
    let rt = drive(
        VW,
        VW,
        ".a{width:100px;height:50px;background-color:rgb(255,0,0);transform:rotate(90deg)}",
        &view(vec![el(10, "a", vec![])]),
    );
    let (x, y, w, h) = geom(&rt, at(&rt, &[0]));
    let t = *transforms(&rt)
        .first()
        .expect("a PushTransform for `transform`");
    let bounds = mapped_bounds(t, Rect::new(Point::new(x, y), Size::new(w, h)));
    approx(bounds.origin.x, 25.0, "painted bbox left");
    approx(bounds.origin.y, -25.0, "painted bbox top");
    approx(bounds.size.width, 50.0, "painted bbox width");
    approx(bounds.size.height, 100.0, "painted bbox height");
    let corner = t.apply(Point::new(x, y));
    approx(corner.x, 75.0, "the untransformed top-left paints at x=75");
    approx(
        corner.y,
        -25.0,
        "the untransformed top-left paints at y=-25",
    );
}

#[doc = include_str!("spec_ch4_visual.doc/transform_functions_compose_left_to_right.md")]
#[test]
fn transform_functions_compose_left_to_right() {
    let rt = drive(
        VW,
        VW,
        ".a{width:50px;height:20px;background-color:rgb(255,0,0);\
            transform-origin:0 0;transform:translate(100px, 0) scale(2)}",
        &view(vec![el(10, "a", vec![])]),
    );
    let (x, y, w, h) = geom(&rt, at(&rt, &[0]));
    let t = *transforms(&rt)
        .first()
        .expect("a PushTransform for `transform`");
    let bounds = mapped_bounds(t, Rect::new(Point::new(x, y), Size::new(w, h)));
    approx(bounds.origin.x, 100.0, "x maps to 100 + 2x, not 2(x + 100)");
    approx(bounds.size.width, 100.0, "the scaled width");
}

#[doc = include_str!("spec_ch4_visual.doc/a_transform_establishes_the_containing_block_for_fixed_descendants.md")]
#[test]
fn a_transform_establishes_the_containing_block_for_fixed_descendants() {
    // Border box at (190, 90) sized 320x220 with a 10px border ⇒ padding box at
    // (200, 100) sized 300x200.
    let rt = drive(
        VW,
        VW,
        ".anc{position:absolute;left:190px;top:90px;width:300px;height:200px;\
             border-top-width:10px;border-right-width:10px;border-bottom-width:10px;\
             border-left-width:10px;border-top-style:solid;border-right-style:solid;\
             border-bottom-style:solid;border-left-style:solid;\
             transform:translateX(0px)} \
         .fx{position:fixed;top:0;left:0;width:50%;height:50px}",
        &view(vec![el(10, "anc", vec![el(100, "fx", vec![])])]),
    );
    let (x, y, w, h) = geom(&rt, at(&rt, &[0, 0]));
    approx(
        x,
        200.0,
        "fixed x resolves against the transformed padding box",
    );
    approx(
        y,
        100.0,
        "fixed y resolves against the transformed padding box",
    );
    approx(w, 150.0, "50% of the 300px padding box");
    approx(h, 50.0, "explicit height");
}

#[doc = include_str!("spec_ch4_visual.doc/a_transform_extends_but_does_not_shrink_scrollable_overflow.md")]
#[test]
fn a_transform_extends_but_does_not_shrink_scrollable_overflow() {
    let mut rt = drive(
        VW,
        VW,
        ".sc{overflow:auto;width:200px;height:200px} \
         .ch{width:400px;height:400px;transform-origin:0 0;transform:scale(0.5)}",
        &view(vec![el(10, "sc", vec![el(100, "ch", vec![])])]),
    );
    let sc = at(&rt, &[0]);
    rt.set_scroll_top(sc, 10_000.0);
    approx(
        rt.scroll_top(sc),
        200.0,
        "the pre-transform 400px bounds still drive the scrollable extent",
    );
}

#[doc = include_str!("spec_ch4_visual.doc/overflow_hidden_clips_to_the_padding_box.md")]
#[test]
fn overflow_hidden_clips_to_the_padding_box() {
    let rt = drive(
        VW,
        VW,
        ".a{overflow:hidden;width:100px;padding:20px;height:100px;\
            background-color:rgb(0,255,0)} \
         .ch{width:200px;height:10px;background-color:rgb(255,0,0)}",
        &view(vec![el(10, "a", vec![el(100, "ch", vec![])])]),
    );
    let (ax, ay, _, _) = geom(&rt, at(&rt, &[0]));
    let clips = clip_bounds(&rt);
    let clip = *clips
        .first()
        .expect("an `overflow: hidden` box pushes a clip");
    approx(
        clip.origin.x,
        ax,
        "the clip starts at the padding-box left edge",
    );
    approx(
        clip.origin.y,
        ay,
        "the clip starts at the padding-box top edge",
    );
    approx(
        clip.size.width,
        140.0,
        "padding box = 20 + 100 + 20, not the 100px content box",
    );
}

#[doc = include_str!("spec_ch4_visual.doc/overflow_clip_is_not_a_scroll_container_but_hidden_is.md")]
#[test]
fn overflow_clip_is_not_a_scroll_container_but_hidden_is() {
    let css = ".h,.c{width:100px;height:100px} .h{overflow:hidden} .c{overflow:clip} \
               .in{width:200px;height:200px}";
    let v = || {
        view(vec![
            el(10, "h", vec![el(100, "in", vec![])]),
            el(11, "c", vec![el(101, "in", vec![])]),
        ])
    };
    let mut rt = drive(VW, VW, css, &v());
    let hidden = at(&rt, &[0]);
    let clip = at(&rt, &[1]);
    rt.set_scroll_top(hidden, 50.0);
    rt.set_scroll_top(clip, 50.0);
    approx(
        rt.scroll_top(hidden),
        50.0,
        "`overflow: hidden` scrolls programmatically",
    );
    approx(
        rt.scroll_top(clip),
        0.0,
        "`overflow: clip` is not a scroll container",
    );
}

#[doc = include_str!("spec_ch4_visual.doc/overlapping_corner_radii_are_reduced_uniformly.md")]
#[test]
fn overlapping_corner_radii_are_reduced_uniformly() {
    let rt = drive(
        VW,
        VW,
        ".a{width:60px;height:50px;background-color:rgb(0,0,255);\
            border-top-left-radius:40px;border-top-right-radius:40px;\
            border-bottom-left-radius:10px;border-bottom-right-radius:10px;\
            box-shadow:0 0 0 rgb(255,0,0)}",
        &view(vec![el(10, "a", vec![])]),
    );
    let r = shadow_radii(&rt);
    // f = min(60/80, 60/20, 50/50, 50/50) = 0.75.
    approx(r.top_left.x, 30.0, "top-left reduced by f");
    approx(r.top_right.x, 30.0, "top-right reduced by f");
    approx(
        r.bottom_left.x,
        7.5,
        "the bottom corners were already small enough, and are reduced anyway",
    );
    approx(r.bottom_right.x, 7.5, "bottom-right reduced by f");
}

#[doc = include_str!("spec_ch4_visual.doc/logical_block_start_maps_by_writing_mode.md")]
#[test]
fn logical_block_start_maps_by_writing_mode() {
    let run = |mode: &str| -> (UiRuntime, (f32, f32, f32, f32), (f32, f32, f32, f32)) {
        let css = format!(
            ".p{{writing-mode:{mode};width:300px;height:300px;padding:0}} \
             .a{{width:100px;height:100px;margin-block-start:20px}}"
        );
        let rt = drive(
            VW,
            VW,
            &css,
            &view(vec![el(10, "p", vec![el(100, "a", vec![])])]),
        );
        let p = geom(&rt, at(&rt, &[0]));
        let a = geom(&rt, at(&rt, &[0, 0]));
        (rt, p, a)
    };

    let (_, _, h) = run("horizontal-tb");
    approx(h.0, 0.0, "horizontal-tb: x");
    approx(h.1, 20.0, "horizontal-tb: block-start is the top");

    let (_, p_rl, rl) = run("vertical-rl");
    approx(
        rl.0 + rl.2,
        p_rl.0 + p_rl.2 - 20.0,
        "vertical-rl: block-start is the right content edge",
    );

    let (_, _, lr) = run("vertical-lr");
    approx(lr.0, 20.0, "vertical-lr: block-start is the left");
    approx(lr.1, 0.0, "vertical-lr: y");
}

#[doc = include_str!("spec_ch4_visual.doc/logical_inline_start_depends_on_direction.md")]
#[test]
fn logical_inline_start_depends_on_direction() {
    let run = |dir: &str| -> (f32, f32) {
        let css = format!(
            ".p{{direction:{dir};width:300px;height:300px}} \
             .a{{width:100px;height:100px;padding-inline-start:30px}}"
        );
        let rt = drive(
            VW,
            VW,
            &css,
            &view(vec![el(10, "p", vec![el(100, "a", vec![])])]),
        );
        let a = at(&rt, &[0, 0]);
        (
            support::px_of(&rt, a, PropertyId::PaddingLeft).unwrap_or(0.0),
            support::px_of(&rt, a, PropertyId::PaddingRight).unwrap_or(0.0),
        )
    };
    let (l, r) = run("ltr");
    approx(l, 30.0, "ltr: padding-left");
    approx(r, 0.0, "ltr: padding-right");
    let (l, r) = run("rtl");
    approx(l, 0.0, "rtl: padding-left");
    approx(r, 30.0, "rtl: padding-right");
}

#[doc = include_str!("spec_ch4_visual.doc/an_absolute_containing_block_is_the_ancestors_padding_box.md")]
#[test]
fn an_absolute_containing_block_is_the_ancestors_padding_box() {
    let rt = drive(
        VW,
        VW,
        ".anc{position:relative;width:100px;height:100px;padding:20px;\
             border-top-width:10px;border-right-width:10px;border-bottom-width:10px;\
             border-left-width:10px;border-top-style:solid;border-right-style:solid;\
             border-bottom-style:solid;border-left-style:solid} \
         .d{position:absolute;top:0;left:0;width:100%;height:25%}",
        &view(vec![el(10, "anc", vec![el(100, "d", vec![])])]),
    );
    let (x, y, w, h) = geom(&rt, at(&rt, &[0, 0]));
    approx(
        x,
        10.0,
        "origin at the padding edge, inside the 10px border",
    );
    approx(
        y,
        10.0,
        "origin at the padding edge, inside the 10px border",
    );
    approx(w, 140.0, "100% of the 140px padding box");
    approx(h, 35.0, "25% of the 140px padding box");
}
