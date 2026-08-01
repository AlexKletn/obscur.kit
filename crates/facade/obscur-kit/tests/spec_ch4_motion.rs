#![doc = include_str!("spec_ch4_motion.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch4_support/mod.rs"]
mod support;

use obscur_kit::bootstrap::CssPipeline;
use obscur_kit::css::source::CssSource;
use obscur_kit::identity::NodeId;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::selector::PseudoState;
use obscur_kit::view::view::View;
use support::{approx, at, el, geom, runtime, view};

const VW: f32 = 1000.0;

#[doc = include_str!("spec_ch4_motion.doc/Host.md")]
struct Host {
    rt: UiRuntime,
    css: CssPipeline,
    v: View,
}

impl Host {
    fn new(css_text: &str, v: View) -> Self {
        let mut rt = runtime(VW, VW);
        let mut css = CssPipeline::new();
        css.install(&mut rt, &[CssSource::anonymous(css_text)]);
        let mut host = Self { rt, css, v };
        host.frame();
        host
    }

    fn frame(&mut self) {
        let sheets = self.css.sheets().to_vec();
        let _ = self.rt.render_now(&self.v, &sheets);
    }

    fn tick(&mut self, dt: f32) {
        self.rt.advance_motion(dt);
        self.frame();
    }

    fn node(&self, path: &[usize]) -> NodeId {
        at(&self.rt, path)
    }

    fn width(&self, path: &[usize]) -> f32 {
        geom(&self.rt, self.node(path)).2
    }

    fn opacity(&self, path: &[usize]) -> Option<f32> {
        self.rt.computed_style(self.node(path))?.opacity()
    }

    fn hover(&mut self, path: &[usize], on: bool) {
        let node = self.node(path);
        if on {
            self.rt.set_pseudo_state(node, PseudoState::HOVER);
        } else {
            self.rt.set_pseudo_state(node, PseudoState::empty());
        }
        self.frame();
    }
}

fn one_box() -> View {
    view(vec![el(10, "box", vec![])])
}

#[doc = include_str!("spec_ch4_motion.doc/transitions_outrank_important_author_declarations.md")]
#[test]
fn transitions_outrank_important_author_declarations() {
    let mut h = Host::new(
        ".box { width: 0px; height: 20px; transition: width 1s linear }\n\
         .box:hover { width: 100px !important }\n",
        one_box(),
    );
    h.hover(&[0], true);
    approx(h.width(&[0]), 0.0, "t=0: the transition starts from 0px");
    h.tick(0.5);
    approx(
        h.width(&[0]),
        50.0,
        "t=0.5s: mid-transition, not pinned by !important",
    );
    h.tick(0.5);
    approx(h.width(&[0]), 100.0, "t=1s: settled");
}

#[doc = include_str!("spec_ch4_motion.doc/a_reversed_transition_is_shortened_by_its_progress.md")]
#[test]
fn a_reversed_transition_is_shortened_by_its_progress() {
    let mut h = Host::new(
        ".box { width: 0px; height: 20px; transition: width 1s linear; \
         transition-delay: 0s }\n\
         .box:hover { width: 100px }\n",
        one_box(),
    );
    h.hover(&[0], true);
    h.tick(0.3);
    approx(h.width(&[0]), 30.0, "t=0.3s on the way out");

    h.hover(&[0], false);
    h.tick(0.15);
    approx(
        h.width(&[0]),
        15.0,
        "t=0.45s: the reverse runs 30px -> 0px over 0.3s",
    );
    h.tick(0.16);
    approx(
        h.width(&[0]),
        0.0,
        "t=0.61s: the reverse already ended at t=0.6s",
    );
}

#[doc = include_str!("spec_ch4_motion.doc/a_negative_transition_delay_starts_part_way_through.md")]
#[test]
fn a_negative_transition_delay_starts_part_way_through() {
    let mut h = Host::new(
        ".box { width: 0px; height: 20px; transition: width 2s linear -0.5s }\n\
         .box:hover { width: 100px }\n",
        one_box(),
    );
    h.hover(&[0], true);
    approx(
        h.width(&[0]),
        25.0,
        "t=0: already 0.5s into a 2s play cycle",
    );
    h.tick(1.0);
    approx(h.width(&[0]), 75.0, "t=1s");
    h.tick(0.55);
    approx(h.width(&[0]), 100.0, "the transition completed at t=1.5s");
}

#[doc = include_str!("spec_ch4_motion.doc/backwards_fill_uses_the_to_keyframe_when_the_direction_is_reversed.md")]
#[test]
fn backwards_fill_uses_the_to_keyframe_when_the_direction_is_reversed() {
    let mut h = Host::new(
        "@keyframes a { from { opacity: 0 } to { opacity: 1 } }\n\
         .box { width: 10px; height: 10px; animation: a 1s 1s alternate-reverse backwards }\n",
        one_box(),
    );
    h.tick(0.5);
    let got = h.opacity(&[0]);
    assert_eq!(
        got,
        Some(1.0),
        "inside the delay, an alternate-reverse animation fills from `to`"
    );
}

#[doc = include_str!("spec_ch4_motion.doc/alternate_iteration_parity_counts_from_one.md")]
#[test]
fn alternate_iteration_parity_counts_from_one() {
    let mut h = Host::new(
        "@keyframes a { from { width: 0px } to { width: 100px } }\n\
         .box { height: 10px; animation: a 1s linear 3 alternate }\n",
        one_box(),
    );
    h.tick(0.25);
    approx(h.width(&[0]), 25.0, "t=0.25s: iteration 1 is odd -> normal");
    h.tick(1.0);
    approx(
        h.width(&[0]),
        75.0,
        "t=1.25s: iteration 2 is even -> reversed",
    );
    h.tick(1.0);
    approx(h.width(&[0]), 25.0, "t=2.25s: iteration 3 is odd -> normal");
}

#[doc = include_str!("spec_ch4_motion.doc/forwards_fill_uses_the_end_of_the_last_completed_iteration.md")]
#[test]
fn forwards_fill_uses_the_end_of_the_last_completed_iteration() {
    let mut h = Host::new(
        "@keyframes a { from { width: 0px } to { width: 100px } }\n\
         .odd { height: 10px; animation: a 1s linear 3 alternate forwards }\n\
         .even { height: 10px; animation: a 1s linear 2 alternate forwards }\n",
        view(vec![el(10, "odd", vec![]), el(11, "even", vec![])]),
    );
    h.tick(5.0);
    approx(
        h.width(&[0]),
        100.0,
        "3 iterations: the last is normal, ending at `to`",
    );
    approx(
        h.width(&[1]),
        0.0,
        "2 iterations: the last is reversed, ending at `from`",
    );
}

#[doc = include_str!("spec_ch4_motion.doc/color_interpolation_is_premultiplied.md")]
#[test]
fn color_interpolation_is_premultiplied() {
    let mut h = Host::new(
        ".box { width: 10px; height: 10px; color: rgb(255 0 0 / 100%); \
         transition: color 1s linear }\n\
         .box:hover { color: rgb(0 0 255 / 0%) }\n",
        one_box(),
    );
    h.hover(&[0], true);
    h.tick(0.5);
    let node = h.node(&[0]);
    let c = support::color_of(&h.rt, node).expect("the box has a computed color");
    approx(c.a, 0.5, "alpha lerps to 50%");
    approx(c.r, 1.0, "premultiplied: full-strength red survives");
    approx(c.g, 0.0, "green stays 0");
    approx(
        c.b,
        0.0,
        "blue is still fully transparent, so it contributes nothing",
    );
}
