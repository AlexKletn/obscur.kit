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

#[test]
fn percentage_keyframes_follow_the_box_as_its_width_animates() {
    use obscur_kit::geometry::Point;
    use obscur_kit::scene::command::PaintCommand;

    let mut host = Host::new(
        "@keyframes move {
            from { width: 100px; transform: none; }
            to { width: 200px; transform: translate(calc(50% + 10px), 50%); }
         }
         .box { height: 40px; background-color: red; transform-origin: 0 0;
                animation: move 1s linear forwards; }",
        one_box(),
    );

    for (dt, width, x, y) in [(0.5, 150.0, 42.5, 10.0), (0.5, 200.0, 110.0, 20.0)] {
        host.tick(dt);
        approx(host.width(&[0]), width, "animated box width");
        let snapshot = host.rt.snapshot();
        let position = snapshot
            .display_list()
            .commands()
            .iter()
            .find_map(|command| match command {
                PaintCommand::PushTransform(transform) => Some(transform.apply(Point::ORIGIN)),
                _ => None,
            })
            .expect("animated translation");
        approx(position.x, x, "horizontal keyframe translation");
        approx(position.y, y, "vertical keyframe translation");
    }
}

#[test]
fn shared_element_starts_at_the_source_on_the_first_destination_frame() {
    use obscur_kit::geometry::{Point, Rect, Size};
    use obscur_kit::motion::track::Easing;
    use obscur_kit::motion::transition::SharedElementKey;
    use obscur_kit::scene::command::PaintCommand;

    let mut host = Host::new(
        ".old { position: absolute; left: 20px; top: 30px; width: 60px; height: 40px; background-color: red }
         .new { position: absolute; left: 320px; top: 80px; width: 120px; height: 80px; background-color: blue }",
        view(vec![el(10, "old", vec![])]),
    );
    let source = host.node(&[0]);
    assert!(host.rt.queue_shared_element(
        source,
        SharedElementKey::new("avatar"),
        1.0,
        Easing::Linear
    ));
    host.v = view(vec![
        el(20, "new", vec![]).with_shared_element_key("avatar"),
    ]);
    host.frame();
    let destination = host.node(&[0]);
    assert_ne!(source, destination);
    let layout = Rect::new(Point::new(320.0, 80.0), Size::new(120.0, 80.0));
    let painted = |rt: &UiRuntime| {
        let snapshot = rt.snapshot();
        snapshot
            .display_list()
            .commands()
            .iter()
            .find_map(|command| match command {
                PaintCommand::PushTransform(transform) => Some(transform.map_rect(layout)),
                _ => None,
            })
            .unwrap_or(layout)
    };
    assert_eq!(host.rt.layout_boxes()[&destination].bounds, layout);
    assert_eq!(
        painted(&host.rt),
        Rect::new(Point::new(20.0, 30.0), Size::new(60.0, 40.0))
    );
    let target_at =
        |rt: &UiRuntime, x, y| rt.hit_test(Point::new(x, y)).target().map(|hit| hit.node());
    assert_eq!(target_at(&host.rt, 35.0, 45.0), Some(destination));
    assert_ne!(target_at(&host.rt, 350.0, 100.0), Some(destination));
    let semantic_bounds =
        |rt: &UiRuntime| rt.semantic_snapshot().get(destination).unwrap().bounds();
    assert_eq!(semantic_bounds(&host.rt), Some(painted(&host.rt)));
    assert!(host.rt.has_pending_frame());
    host.tick(0.5);
    assert_eq!(
        painted(&host.rt),
        Rect::new(Point::new(170.0, 55.0), Size::new(90.0, 60.0))
    );
    assert_eq!(semantic_bounds(&host.rt), Some(painted(&host.rt)));
    assert_eq!(target_at(&host.rt, 190.0, 70.0), Some(destination));
    assert_ne!(target_at(&host.rt, 35.0, 45.0), Some(destination));
    host.tick(0.5);
    assert_eq!(painted(&host.rt), layout);
    assert_eq!(semantic_bounds(&host.rt), Some(layout));
    assert_eq!(target_at(&host.rt, 350.0, 100.0), Some(destination));
}

#[test]
fn retargeting_a_shared_element_preserves_the_displayed_frame_and_css_transform() {
    use obscur_kit::geometry::{Point, Rect, Size};
    use obscur_kit::motion::track::Easing;
    use obscur_kit::motion::transition::SharedElementKey;
    use obscur_kit::scene::command::PaintCommand;

    let mut host = Host::new(
        ".first { position:absolute; left:20px; top:30px; width:60px; height:40px; background:red }
         .second { position:absolute; left:320px; top:80px; width:120px; height:80px; background:blue }
         .third { position:absolute; left:500px; top:30px; width:80px; height:40px; background:green;
                  transform-origin:0 0; transform:translateX(10px) scale(1.5, 2); }",
        view(vec![el(10, "first", vec![])]),
    );
    let painted = |host: &Host| {
        let layout = host.rt.layout_boxes()[&host.node(&[0])].bounds;
        let snapshot = host.rt.snapshot();
        snapshot
            .display_list()
            .commands()
            .iter()
            .find_map(|command| match command {
                PaintCommand::PushTransform(transform) => Some(transform.map_rect(layout)),
                _ => None,
            })
            .unwrap_or(layout)
    };
    let source = host.node(&[0]);
    assert!(host.rt.queue_shared_element(
        source,
        SharedElementKey::new("shared"),
        1.0,
        Easing::Linear
    ));
    host.v = view(vec![
        el(20, "second", vec![]).with_shared_element_key("shared"),
    ]);
    host.frame();
    host.tick(0.5);
    let before = painted(&host);
    assert_eq!(
        before,
        Rect::new(Point::new(170.0, 55.0), Size::new(90.0, 60.0))
    );

    let source = host.node(&[0]);
    assert!(host.rt.queue_shared_element(
        source,
        SharedElementKey::new("shared"),
        1.0,
        Easing::Linear
    ));
    host.v = view(vec![
        el(30, "third", vec![]).with_shared_element_key("shared"),
    ]);
    host.frame();
    assert_eq!(painted(&host), before);
    host.tick(0.5);
    assert_eq!(
        painted(&host),
        Rect::new(Point::new(340.0, 42.5), Size::new(105.0, 70.0))
    );
    host.tick(0.5);
    assert_eq!(
        painted(&host),
        Rect::new(Point::new(510.0, 30.0), Size::new(120.0, 80.0))
    );
}

#[test]
fn animated_opacity_uses_the_presented_stacking_context_for_hits() {
    use obscur_kit::geometry::Point;

    let mut h = Host::new(
        "@keyframes fade { from { opacity:1 } to { opacity:0.5 } }
         .parent,.child,.sibling{position:absolute;left:0;top:0;width:80px;height:60px}
         .parent{animation:fade 1s linear}
         .child{z-index:100;background-color:red}
         .sibling{z-index:1;background-color:blue}",
        view(vec![el(
            9,
            "wrap",
            vec![
                el(1, "parent", vec![el(2, "child", vec![])]),
                el(3, "sibling", vec![]),
            ],
        )]),
    );
    let child = h.node(&[0, 0, 0]);
    let sibling = h.node(&[0, 1]);
    let point = Point::new(20.0, 20.0);
    assert_eq!(
        h.rt.hit_test(point).target().map(|hit| hit.node()),
        Some(child)
    );
    h.tick(0.5);
    assert_eq!(
        h.rt.hit_test(point).target().map(|hit| hit.node()),
        Some(sibling)
    );
}
