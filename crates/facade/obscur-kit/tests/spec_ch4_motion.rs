//! **SPECIFICATION-DERIVED** transition and animation cases for obscur.kit §4.17
//! ("CSS `transition`, `@keyframes` and `animation-*` are parsed as a
//! compatibility/declarative frontend and compiled into native timelines";
//! "time-driven motion"; "interruption; reversal"; "style interpolation").
//!
//! Every expected number below was produced by an agent reading the specification
//! — obscur.kit §4 and the W3C css-transitions-1 / css-animations-1 /
//! css-cascade-5 / css-color-4 text it normatively cites — with **no access to
//! this repository's source**. A failure here is a defect report about the
//! engine, and must not be "corrected" by editing the expected value.
//!
//! Translation note: the deriver triggers each transition by "adding class `on`".
//! Here the trigger is `:hover` via [`UiRuntime::set_pseudo_state`], which is the
//! style-change mechanism this workspace exposes. Nothing in any case depends on
//! *which* style change fires the transition.
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

/// A runtime with `css` installed through the real CSS pipeline, plus the view.
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

/// css-cascade-5 §6.1: "The precedence of the various origins is, in descending
/// order: / Transition declarations [css-transitions-1] / Important user agent
/// declarations / Important user declarations / Important author declarations".
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

/// css-transitions-1 §4: "reversing shortening factor is the absolute value,
/// clamped to the range [0, 1], of the sum of: the output of the timing function
/// of the old transition at the time of the style change event, times the
/// reversing shortening factor of the old transition / 1 minus the reversing
/// shortening factor of the old transition." and "end time is the start time plus
/// the product of the matching transition duration and the new transition's
/// reversing shortening factor".
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

/// css-transitions-1 §2.4: "If the value for transition-delay is a negative time
/// offset then the transition will execute the moment the property is changed, but
/// will appear to have begun execution at the specified offset. That is, the
/// transition will appear to begin part-way through its play cycle."
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

/// css-animations-1 §3.9: "backwards / During the period defined by
/// animation-delay, the animation will apply the property values defined in the
/// keyframe that will start the first iteration of the animation. These are either
/// the values of the from keyframe (when animation-direction is normal or
/// alternate) or those of the to keyframe (when animation-direction is reverse or
/// alternate-reverse)."
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

/// css-animations-1 §3.6: "alternate / The animation cycle iterations that are odd
/// counts are played in the normal direction, and the animation cycle iterations
/// that are even counts are played in a reverse direction." and "Note: For the
/// purpose of determining whether an iteration is even or odd, iterations start
/// counting from 1."
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

/// css-animations-1 §3.9: "forwards / After the animation ends (as determined by
/// its animation-iteration-count), the animation will apply the property values
/// for the time the animation ended. When animation-iteration-count is an integer
/// greater than zero, the values applied will be those for the end of the last
/// completed iteration of the animation (rather than the values for the start of
/// the iteration that would be next)."
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

/// css-color-4 §13.3 Interpolating with Alpha: "For rectangular orthogonal color
/// coordinate systems, all component values are multiplied by the alpha value."
/// and "otherwise, each component which had been premultiplied is divided by the
/// interpolated alpha value."
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
