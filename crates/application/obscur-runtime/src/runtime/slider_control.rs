#![doc = include_str!("../../docs/api/runtime/slider_control.doc/_module.md")]

use super::slider_grab::Grab;

use obscur_geometry::{Point, Rect};
use obscur_identity::NodeId;
use obscur_tree::tree::RetainedTree;
use obscur_view::view::{SLIDER_HIGH, SLIDER_LOW, SLIDER_RANGED, SliderScale, ViewValue};

use super::AncestorChain;
use crate::orchestration::LayoutBoxes;

#[doc = include_str!("../../docs/api/runtime/slider_control.doc/THUMB_INSET.md")]
const THUMB_INSET: f32 = 8.0;

/// A slider found in the tree, with everything a press needs to know about it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SliderControl {
    /// The node that IS the slider — not necessarily the one that was hit.
    pub(crate) node: NodeId,
    /// The bounds and step it offers.
    pub(crate) scale: SliderScale,
    /// Its value, or the low end of its range.
    pub(crate) low: f32,
    /// The high end of its range; equal to the low end on a single slider.
    pub(crate) high: f32,
    /// Whether it picks a range rather than a single value.
    pub(crate) ranged: bool,
    /// Its laid-out box.
    pub(crate) track: Rect,
}

impl SliderControl {
    #[doc = include_str!("../../docs/api/runtime/slider_control.doc/enclosing.md")]
    pub(crate) fn enclosing(
        tree: &RetainedTree,
        boxes: &LayoutBoxes,
        node: NodeId,
    ) -> Option<Self> {
        // The press lands on whatever the hit test found, which for a slider is
        // usually the canvas inside it rather than the control. So the walk goes
        // up: the nearest ancestor that declares a slider's `low` IS the slider.
        AncestorChain::of(tree, Some(node))
            .into_iter()
            .find_map(|at| {
                let retained = tree.get(at)?;
                let props = retained.props();
                let low = number(props, SLIDER_LOW)?;
                let bounds = boxes.get(&at)?;
                Some(Self {
                    node: at,
                    scale: SliderScale::read(props),
                    low,
                    high: number(props, SLIDER_HIGH).unwrap_or(low),
                    ranged: flag(props, SLIDER_RANGED),
                    track: Rect::new(bounds.origin(), bounds.size()),
                })
            })
    }

    /// Where along the track a point falls, from 0 to 1, clamped.
    pub(crate) fn fraction_at(&self, point: Point) -> f32 {
        // The same inset the painter uses, so the value under the pointer is the
        // value the thumb is drawn at. A track measured to the box's edges would
        // put every fraction slightly off, and the error would be worst exactly
        // where a user aims most carefully: the two ends.
        let inset = THUMB_INSET.min(self.track.size.width / 2.0);
        let left = self.track.origin.x + inset;
        let usable = self.track.size.width - inset * 2.0;
        if usable <= 0.0 {
            return 0.0;
        }
        ((point.x - left) / usable).clamp(0.0, 1.0)
    }

    #[doc = include_str!("../../docs/api/runtime/slider_control.doc/grab_for.md")]
    pub(crate) fn grab_for(&self, point: Point) -> Grab {
        if !self.ranged {
            return Grab::Low;
        }
        // The nearer end, and the low one on a tie. A press exactly between two
        // thumbs has to move one of them, and moving the low one is what lets a
        // collapsed range — both ends together — be opened by dragging right.
        let at = self.fraction_at(point);
        let (to_low, to_high) = (
            (at - self.scale.fraction_of(self.low)).abs(),
            (at - self.scale.fraction_of(self.high)).abs(),
        );
        if to_high < to_low {
            Grab::High
        } else {
            Grab::Low
        }
    }

    /// The ends after dragging `grab` to this point.
    pub(crate) fn moved_to(&self, grab: Grab, point: Point) -> (f32, f32) {
        let value = self.scale.value_at(self.fraction_at(point));
        self.with(grab, value)
    }

    #[doc = include_str!("../../docs/api/runtime/slider_control.doc/nudged.md")]
    pub(crate) fn nudged(&self, grab: Grab, steps: f32) -> (f32, f32) {
        // A free slider has no step to move by, so it moves by a hundredth of
        // its span — the smallest amount that still crosses the whole track in a
        // reasonable number of presses.
        let step = self.scale.step.unwrap_or(self.scale.span() / 100.0);
        let from = match grab {
            Grab::Low => self.low,
            Grab::High => self.high,
        };
        self.with(grab, self.scale.quantize(from + steps * step))
    }

    #[doc = include_str!("../../docs/api/runtime/slider_control.doc/set.md")]
    pub(crate) fn set(&self, grab: Grab, value: f32) -> (f32, f32) {
        self.with(grab, self.scale.quantize(value))
    }

    #[doc = include_str!("../../docs/api/runtime/slider_control.doc/with.md")]
    fn with(&self, grab: Grab, value: f32) -> (f32, f32) {
        // A single slider is a range of one point, and both of its ends move
        // together. Applying the no-crossing rule to it would clamp its value
        // against ITSELF — a thumb that could never move at all.
        if !self.ranged {
            return (value, value);
        }
        // On a range the ends never cross. A thumb dragged past its neighbour
        // stops there rather than swapping: swapping mid-drag would leave the
        // pointer holding the other thumb, which is a control that fights its
        // user.
        match grab {
            Grab::Low => (value.min(self.high), self.high),
            Grab::High => (self.low, value.max(self.low)),
        }
    }
}

/// A numeric property, whichever way it was written.
fn number(
    props: &[obscur_view::view::ViewProperty],
    name: obscur_identity::InternedSymbol,
) -> Option<f32> {
    props
        .iter()
        .find(|p| p.name() == name)
        .and_then(|p| match p.value() {
            #[allow(clippy::cast_possible_truncation)]
            ViewValue::Float(v) => Some(*v as f32),
            #[allow(clippy::cast_precision_loss)]
            ViewValue::Int(v) => Some(*v as f32),
            _ => None,
        })
        .filter(|v| v.is_finite())
}

/// A boolean property, defaulting to false.
fn flag(props: &[obscur_view::view::ViewProperty], name: obscur_identity::InternedSymbol) -> bool {
    props
        .iter()
        .find(|p| p.name() == name)
        .and_then(|p| match p.value() {
            ViewValue::Bool(v) => Some(*v),
            _ => None,
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{Grab, SliderControl};
    use obscur_geometry::{Point, Rect, Size};
    use obscur_identity::NodeId;
    use obscur_view::view::SliderScale;

    fn control(low: f32, high: f32, ranged: bool) -> SliderControl {
        SliderControl {
            node: NodeId::from_raw(1),
            scale: SliderScale::new(0.0, 100.0),
            low,
            high,
            ranged,
            // 116 wide: 100 usable between the two 8px insets.
            track: Rect::new(Point::new(0.0, 0.0), Size::new(116.0, 20.0)),
        }
    }

    fn at(x: f32) -> Point {
        Point::new(x, 10.0)
    }

    #[test]
    fn the_fraction_is_measured_between_the_insets_not_the_edges() {
        // The value under the pointer must be the value the thumb is drawn at,
        // and the error of measuring to the edges is worst exactly where a user
        // aims most carefully: the two ends.
        let slider = control(0.0, 0.0, false);
        assert!(
            (slider.fraction_at(at(8.0))).abs() < 0.001,
            "the left thumb"
        );
        assert!(
            (slider.fraction_at(at(108.0)) - 1.0).abs() < 0.001,
            "the right"
        );
        assert!(
            (slider.fraction_at(at(58.0)) - 0.5).abs() < 0.001,
            "the middle"
        );
    }

    #[test]
    fn a_press_beyond_either_end_clamps_rather_than_running_off() {
        let slider = control(0.0, 0.0, false);
        assert_eq!(slider.fraction_at(at(-500.0)), 0.0);
        assert_eq!(slider.fraction_at(at(9_000.0)), 1.0);
    }

    #[test]
    fn a_drag_sets_the_value_the_pointer_is_over() {
        let slider = control(0.0, 0.0, false);
        let (low, _) = slider.moved_to(Grab::Low, at(58.0));
        assert!((low - 50.0).abs() < 0.01, "half the scale: {low}");
    }

    #[test]
    fn a_ranged_press_grabs_the_nearer_thumb() {
        let slider = control(20.0, 80.0, true);
        assert_eq!(slider.grab_for(at(8.0 + 25.0)), Grab::Low);
        assert_eq!(slider.grab_for(at(8.0 + 75.0)), Grab::High);
    }

    #[test]
    fn a_collapsed_range_opens_by_dragging_and_does_not_deadlock() {
        // Both ends together: a press must grab the LOW one, because that is
        // what lets a drag to the right open the range at all.
        let slider = control(50.0, 50.0, true);
        assert_eq!(slider.grab_for(at(58.0)), Grab::Low);
    }

    #[test]
    fn a_single_sliders_value_is_not_clamped_against_itself() {
        // The defect this caught: a single slider is a range of one point, so
        // applying the no-crossing rule to it clamps its value against ITSELF
        // and the thumb can never move.
        let slider = control(0.0, 0.0, false);
        assert_eq!(slider.moved_to(Grab::Low, at(108.0)), (100.0, 100.0));
    }

    #[test]
    fn the_ends_never_cross() {
        // A thumb dragged past its neighbour stops there rather than swapping:
        // swapping mid-drag would leave the pointer holding the other thumb,
        // which is a control that fights its user.
        let slider = control(20.0, 80.0, true);
        let (low, high) = slider.moved_to(Grab::Low, at(8.0 + 95.0));
        assert_eq!((low, high), (80.0, 80.0), "the low end stops at the high");

        let (low, high) = slider.moved_to(Grab::High, at(8.0 + 5.0));
        assert_eq!((low, high), (20.0, 20.0), "and the high end at the low");
    }

    #[test]
    fn an_arrow_moves_by_the_step_when_there_is_one() {
        let mut slider = control(50.0, 50.0, false);
        slider.scale = SliderScale::new(0.0, 100.0).stepped_by(10.0);
        assert_eq!(slider.nudged(Grab::Low, 1.0).0, 60.0);
        assert_eq!(slider.nudged(Grab::Low, -1.0).0, 40.0);
    }

    #[test]
    fn an_arrow_on_a_free_slider_moves_by_a_hundredth_of_the_span() {
        // A free slider has no step to move by, and moving by nothing would be
        // a control the keyboard cannot drive at all.
        let slider = control(50.0, 50.0, false);
        assert!((slider.nudged(Grab::Low, 1.0).0 - 51.0).abs() < 0.01);
    }

    #[test]
    fn an_arrow_at_the_end_stays_there_rather_than_running_past() {
        let slider = control(100.0, 100.0, false);
        assert_eq!(slider.nudged(Grab::Low, 1.0).0, 100.0);
        assert_eq!(control(0.0, 0.0, false).nudged(Grab::Low, -1.0).0, 0.0);
    }

    #[test]
    fn home_and_end_move_a_single_slider_by_the_same_rule_as_everything_else() {
        // The defect this caught: Home wrote the minimum into the LOW end and
        // left the high one where it was — on a single slider, whose two ends
        // are one point, that reported a range. The application then had a
        // slider that had silently become a ranged one.
        let slider = control(60.0, 60.0, false);
        assert_eq!(slider.set(Grab::Low, slider.scale.min), (0.0, 0.0));
        assert_eq!(slider.set(Grab::Low, slider.scale.max), (100.0, 100.0));
    }

    #[test]
    fn end_on_a_range_stops_at_the_neighbouring_thumb() {
        let slider = control(20.0, 80.0, true);
        assert_eq!(slider.set(Grab::Low, 100.0), (80.0, 80.0));
    }

    #[test]
    fn a_track_with_no_room_answers_the_start_rather_than_dividing_by_nothing() {
        let mut slider = control(0.0, 0.0, false);
        slider.track = Rect::new(Point::new(0.0, 0.0), Size::new(0.0, 20.0));
        assert_eq!(slider.fraction_at(at(5.0)), 0.0);
    }
}
