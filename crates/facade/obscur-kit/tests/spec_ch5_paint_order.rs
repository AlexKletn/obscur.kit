#![doc = include_str!("spec_ch5_paint_order.doc/_module.md")]
#![forbid(unsafe_code)]
#![cfg(feature = "render-cpu")]

#[path = "spec_ch5_support/mod.rs"]
mod support;

use std::cell::RefCell;
use std::rc::Rc;

use obscur_kit::diagnostics::{Diagnostic, DiagnosticSink, DiagnosticSubsystem, RecoveryAction};
use obscur_kit::render::{EffectFallbackReporter, RendererId, unsupported_effects};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::scene::clip::Clip;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::scene::paint::{BlendMode, Brush, Paint};
use support::{
    Raster, commands, describe, drive, el, leaves, rasterize, rgb8_of, runtime, txt, view,
};

#[doc = include_str!("spec_ch5_paint_order.doc/VW.md")]
const VW: f32 = 200.0;

#[doc = include_str!("spec_ch5_paint_order.doc/Recorder.md")]
#[derive(Default)]
struct Recorder {
    seen: Rc<RefCell<Vec<Diagnostic>>>,
}

impl DiagnosticSink for Recorder {
    fn emit(&self, diagnostic: &Diagnostic) {
        self.seen.borrow_mut().push(diagnostic.clone());
    }
}

#[doc = include_str!("spec_ch5_paint_order.doc/sc_01_paint_order_within_one_element.md")]
#[test]
fn sc_01_paint_order_within_one_element() {
    let rt = drive(
        VW,
        VW,
        ".a{position:absolute;left:0;top:0;width:100px;height:100px;\
            background-color:rgb(0,0,255);\
            border-top-width:4px;border-right-width:4px;border-bottom-width:4px;\
            border-left-width:4px;border-top-style:solid;border-right-style:solid;\
            border-bottom-style:solid;border-left-style:solid;\
            border-top-color:rgb(0,0,0);border-right-color:rgb(0,0,0);\
            border-bottom-color:rgb(0,0,0);border-left-color:rgb(0,0,0);\
            box-shadow:4px 4px 0 rgb(255,0,0), inset 0 0 0 4px rgb(0,255,0)}",
        &view(vec![el(10, "a", vec![txt("x")])]),
    );

    let ops = leaves(&rt);
    let order: Vec<String> = ops.iter().map(describe).collect();

    // Index of the first leaf satisfying `pred`, or a failure naming the order.
    let find = |what: &str, pred: &dyn Fn(&PaintCommand) -> bool| -> usize {
        ops.iter()
            .position(pred)
            .unwrap_or_else(|| panic!("no {what} in the paint order {order:?}"))
    };

    let outer = find("outer shadow", &|c| {
        matches!(c, PaintCommand::DrawShadow { inset: false, color, .. }
            if rgb8_of(*color) == "rgb(255,0,0)")
    });
    let background = find("background fill", &|c| {
        support::fill_color(c).map(rgb8_of).as_deref() == Some("rgb(0,0,255)")
    });
    let inner = find("inset shadow", &|c| {
        matches!(c, PaintCommand::DrawShadow { inset: true, color, .. }
            if rgb8_of(*color) == "rgb(0,255,0)")
    });
    let border = find("border fill", &|c| {
        support::fill_color(c).map(rgb8_of).as_deref() == Some("rgb(0,0,0)")
    });
    let text = find("text run", &|c| matches!(c, PaintCommand::DrawText { .. }));

    assert!(
        outer < background,
        "the outer shadow is drawn immediately below the background: {order:?}"
    );
    assert!(
        background < inner,
        "the inset shadow is drawn above the background: {order:?}"
    );
    assert!(
        inner < border,
        "the inset shadow is drawn below the borders: {order:?}"
    );
    assert!(
        border < text,
        "the border is drawn below the element's text: {order:?}"
    );
}

#[doc = include_str!("spec_ch5_paint_order.doc/sc_02_outer_shadow_is_clipped_out_of_the_border_box.md")]
#[test]
fn sc_02_outer_shadow_is_clipped_out_of_the_border_box() {
    // The box sits at (20,20) so the 5px spread ring is inside the raster.
    let rt = drive(
        VW,
        VW,
        ".page{position:absolute;left:0;top:0;width:200px;height:200px;\
               background-color:rgb(255,255,255)}\
         .a{position:absolute;left:20px;top:20px;width:100px;height:100px;\
            background-color:rgba(0,0,255,0.5);\
            box-shadow:0 0 0 5px rgb(255,0,0)}",
        &view(vec![el(10, "page", vec![]), el(20, "a", vec![])]),
    );

    let px = rasterize(&rt, VW, VW);

    // Interior: a translucent blue over white. No red may leak through, so the
    // red channel must equal the green channel exactly.
    let (r, g, _, _) = px.at(70, 70);
    assert_eq!(
        r, g,
        "interior pixel is a neutral+blue mix over white; R must equal G \
         (R={r}, G={g}) — a shadow painted under a translucent background tints it"
    );

    // 2px outside the border box, inside the 5px ring: the shadow.
    assert_eq!(
        rgb_at(&px, 18, 70),
        "rgb(255,0,0)",
        "2px left of the box is the shadow ring"
    );
    // 6px outside: past the ring, the white page.
    assert_eq!(
        rgb_at(&px, 14, 70),
        "rgb(255,255,255)",
        "6px left of the box is past the 5px spread"
    );
}

#[doc = include_str!("spec_ch5_paint_order.doc/sc_05_conic_gradient_is_a_scene_brush_and_rasterizes.md")]
#[test]
fn sc_05_conic_gradient_is_a_scene_brush_and_rasterizes() {
    let rt = drive(
        VW,
        VW,
        ".a{position:absolute;left:0;top:0;width:100px;height:100px;\
            background-image:conic-gradient(from 0deg, rgb(255,0,0) 0deg, \
              rgb(255,0,0) 90deg, rgb(0,0,255) 90deg, rgb(0,0,255) 360deg)}",
        &view(vec![el(10, "a", vec![])]),
    );

    let list = commands(&rt);
    let conics = list
        .iter()
        .filter(|c| {
            matches!(
                c,
                PaintCommand::DrawPath {
                    paint: Paint::Fill(Brush::Conic(_)),
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        conics,
        1,
        "one Conic gradient brush reaches the scene; the display list is {:?}",
        list.iter().map(describe).collect::<Vec<_>>()
    );

    let px = rasterize(&rt, VW, VW);
    assert_eq!(rgb_at(&px, 75, 25), "rgb(255,0,0)", "45deg is red");
    assert_eq!(rgb_at(&px, 25, 75), "rgb(0,0,255)", "225deg is blue");
    assert_eq!(rgb_at(&px, 75, 75), "rgb(0,0,255)", "135deg is blue");
}

#[doc = include_str!("spec_ch5_paint_order.doc/sc_06_mix_blend_mode_multiply_exists.md")]
#[test]
fn sc_06_mix_blend_mode_multiply_exists() {
    let rt = drive(
        VW,
        VW,
        ".back{position:absolute;left:0;top:0;width:100px;height:100px;\
               background-color:rgb(0,255,255)}\
         .front{position:absolute;left:25px;top:25px;width:50px;height:50px;\
                background-color:rgb(255,0,0);mix-blend-mode:multiply}",
        &view(vec![el(10, "back", vec![]), el(20, "front", vec![])]),
    );

    let list = commands(&rt);
    let multiplies = list
        .iter()
        .filter(|c| match c {
            PaintCommand::PushLayer(layer) => layer.blend_mode() == BlendMode::Multiply,
            _ => false,
        })
        .count();
    assert_eq!(
        multiplies,
        1,
        "the child's layer carries a multiply blend; the display list is {:?}",
        list.iter().map(describe).collect::<Vec<_>>()
    );

    let px = rasterize(&rt, VW, VW);
    assert_eq!(rgb_at(&px, 10, 10), "rgb(0,255,255)", "outside the child");
    assert_eq!(rgb_at(&px, 30, 30), "rgb(0,0,0)", "cyan x red = black");
    assert_eq!(rgb_at(&px, 50, 50), "rgb(0,0,0)", "cyan x red = black");
}

#[doc = include_str!("spec_ch5_paint_order.doc/sc_08_plain_and_rounded_clips_need_no_offscreen_pass.md")]
#[test]
fn sc_08_plain_and_rounded_clips_need_no_offscreen_pass() {
    let rect_clip = drive(
        VW,
        VW,
        ".c{position:absolute;left:0;top:0;width:50px;height:50px;overflow:hidden}\
         .kid{width:500px;height:500px;background-color:rgb(255,0,0)}",
        &view(vec![el(10, "c", vec![el(11, "kid", vec![])])]),
    );
    assert_eq!(
        pushed_layers(&rect_clip),
        0,
        "(a) a rectangular overflow clip pushes no layer"
    );

    let rounded_clip = drive(
        VW,
        VW,
        ".c{position:absolute;left:0;top:0;width:50px;height:50px;overflow:hidden;\
            border-radius:8px}\
         .kid{width:500px;height:500px;background-color:rgb(255,0,0)}",
        &view(vec![el(10, "c", vec![el(11, "kid", vec![])])]),
    );
    assert_eq!(
        pushed_layers(&rounded_clip),
        0,
        "(b) a rounded clip is a first-class clip (§5.8), not a mask pass"
    );
    assert!(
        commands(&rounded_clip)
            .iter()
            .any(|c| matches!(c, PaintCommand::PushClip(Clip::Shape(_)))),
        "(b) the rounded clip reaches the scene as a typed clip shape"
    );
}

#[doc = include_str!("spec_ch5_paint_order.doc/sc_09_the_scene_snapshot_is_reproducible_and_stays_typed.md")]
#[test]
fn sc_09_the_scene_snapshot_is_reproducible_and_stays_typed() {
    const CSS: &str = ".c{position:absolute;left:0;top:0;width:120px;height:120px;\
                          border-radius:8px;overflow:hidden;\
                          background-image:linear-gradient(rgb(255,0,0),rgb(0,0,255))}\
                       .r{width:60px;height:60px;border-radius:12px;\
                          background-color:rgb(0,255,0)}\
                       .t{transform:rotate(15deg)}";

    let a = drive(
        VW,
        VW,
        CSS,
        &view(vec![el(
            10,
            "c",
            vec![el(11, "r", vec![txt("hello")]), el(12, "t", vec![])],
        )]),
    );
    let b = drive(
        VW,
        VW,
        CSS,
        &view(vec![el(
            10,
            "c",
            vec![el(11, "r", vec![txt("hello")]), el(12, "t", vec![])],
        )]),
    );

    let (left, right) = (commands(&a), commands(&b));
    assert_eq!(
        left.len(),
        right.len(),
        "two renders of one tree produce the same command count"
    );
    for (i, (l, r)) in left.iter().zip(right.iter()).enumerate() {
        assert!(
            l == r,
            "command {i} differs with zero tolerance: {} vs {}",
            describe(l),
            describe(r)
        );
    }

    // The typed-geometry half: the rounded clip stays a typed clip shape (not a
    // flattened 32-segment path) and the rotation stays a Transform command (not
    // pre-multiplied into the geometry).
    assert!(
        left.iter()
            .any(|c| matches!(c, PaintCommand::PushClip(Clip::Shape(_)))),
        "the rounded clip is carried as a typed clip, not flattened"
    );
    assert!(
        left.iter()
            .any(|c| matches!(c, PaintCommand::PushTransform(_))),
        "the rotation is carried as a Transform command, not baked into geometry"
    );
}

#[doc = include_str!("spec_ch5_paint_order.doc/sc_10_an_unsupported_effect_emits_exactly_one_diagnostic.md")]
#[test]
fn sc_10_an_unsupported_effect_emits_exactly_one_diagnostic() {
    let seen: Rc<RefCell<Vec<Diagnostic>>> = Rc::default();
    let recorder = Recorder {
        seen: Rc::clone(&seen),
    };

    let mut rt = runtime(VW, VW);
    let sheet = support::compile(
        ".a{position:absolute;left:0;top:0;width:100px;height:100px;\
            background-color:rgba(0,0,255,0.5);backdrop-filter:blur(8px)}",
    );
    let v = view(vec![el(10, "a", vec![])]);
    let _ = rt.render_now(&v, std::slice::from_ref(&sheet));

    // The CPU backend has no backdrop pass, so the effect cannot be honoured.
    let px = rasterize(&rt, VW, VW);
    let (_, _, b, a) = px.at(50, 50);
    assert!(
        a > 0 && b > 0,
        "the node is still painted without the blur, got a={a} b={b}"
    );

    // Ask the backend that will draw this scene what it cannot honour, and
    // report it — the surface §5.2's sentence names.
    let renderer = CpuRenderer::new(RendererId::from_raw(11));
    let snapshot = rt.snapshot();
    let unsupported = unsupported_effects(&renderer, snapshot.display_list());
    assert!(
        !unsupported.is_empty(),
        "the CPU backend declares the backdrop filter unsupported"
    );

    let mut reporter = EffectFallbackReporter::new();
    reporter.report(&unsupported, &recorder);
    // Reporting the same scene again must not raise it a second time.
    reporter.report(&unsupported, &recorder);

    let diagnostics = seen.borrow();
    assert_eq!(
        diagnostics.len(),
        1,
        "exactly one diagnostic for the unsupported effect, across two reports \
         of the same scene, got {:?}",
        diagnostics
            .iter()
            .map(|d| (d.code, &d.message))
            .collect::<Vec<_>>()
    );
    let d = &diagnostics[0];
    assert_eq!(
        d.subsystem,
        DiagnosticSubsystem::Render,
        "the diagnostic is attributed to the renderer"
    );
    assert_ne!(
        d.recovery,
        RecoveryAction::None,
        "the diagnostic states the recovery action taken (rendered without blur)"
    );
    assert!(
        !d.code.as_str().is_empty(),
        "the diagnostic carries a stable code"
    );
}

#[doc = include_str!("spec_ch5_paint_order.doc/pushed_layers.md")]
fn pushed_layers(rt: &obscur_kit::runtime::UiRuntime) -> usize {
    commands(rt)
        .iter()
        .filter(|c| matches!(c, PaintCommand::PushLayer(_)))
        .count()
}

#[doc = include_str!("spec_ch5_paint_order.doc/rgb_at.md")]
fn rgb_at(px: &Raster, x: u32, y: u32) -> String {
    let (r, g, b, _) = px.at(x, y);
    format!("rgb({r},{g},{b})")
}
