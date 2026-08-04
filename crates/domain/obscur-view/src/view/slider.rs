#![doc = include_str!("slider.doc/_module.md")]

use obscur_identity::InternedSymbol;

use super::{ViewProperty, ViewValue};

#[doc = include_str!("slider.doc/SLIDER_MIN.md")]
pub const SLIDER_MIN: InternedSymbol = InternedSymbol::from_raw(0x0000_0501);
#[doc = include_str!("slider.doc/SLIDER_MAX.md")]
pub const SLIDER_MAX: InternedSymbol = InternedSymbol::from_raw(0x0000_0502);
#[doc = include_str!("slider.doc/SLIDER_STEP.md")]
pub const SLIDER_STEP: InternedSymbol = InternedSymbol::from_raw(0x0000_0503);
#[doc = include_str!("slider.doc/SLIDER_LOW.md")]
pub const SLIDER_LOW: InternedSymbol = InternedSymbol::from_raw(0x0000_0504);
#[doc = include_str!("slider.doc/SLIDER_HIGH.md")]
pub const SLIDER_HIGH: InternedSymbol = InternedSymbol::from_raw(0x0000_0505);
#[doc = include_str!("slider.doc/SLIDER_RANGED.md")]
pub const SLIDER_RANGED: InternedSymbol = InternedSymbol::from_raw(0x0000_0506);

#[doc = include_str!("slider.doc/SliderScale.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SliderScale {
    #[doc = include_str!("slider.doc/min.md")]
    pub min: f32,
    #[doc = include_str!("slider.doc/max.md")]
    pub max: f32,
    #[doc = include_str!("slider.doc/step.md")]
    pub step: Option<f32>,
}

impl Default for SliderScale {
    fn default() -> Self {
        // The scale a control with no stated bounds has: a plain fraction, free
        // rather than stepped. It is what `<slider/>` means with nothing else
        // said, and it is the one range in which a value and its fraction are
        // the same number.
        Self {
            min: 0.0,
            max: 1.0,
            step: None,
        }
    }
}

impl SliderScale {
    #[doc = include_str!("slider.doc/new.md")]
    #[must_use]
    pub fn new(min: f32, max: f32) -> Self {
        Self {
            min,
            max,
            step: None,
        }
    }

    #[doc = include_str!("slider.doc/stepped_by.md")]
    #[must_use]
    pub fn stepped_by(mut self, step: f32) -> Self {
        // A non-positive or non-finite step is no step. Honouring one would
        // divide by it.
        self.step = (step.is_finite() && step > 0.0).then_some(step);
        self
    }

    #[doc = include_str!("slider.doc/span.md")]
    #[must_use]
    pub fn span(&self) -> f32 {
        self.max - self.min
    }

    #[doc = include_str!("slider.doc/is_degenerate.md")]
    #[must_use]
    pub fn is_degenerate(&self) -> bool {
        !self.min.is_finite() || !self.max.is_finite() || self.span() <= 0.0
    }

    #[doc = include_str!("slider.doc/quantize.md")]
    #[must_use]
    pub fn quantize(&self, value: f32) -> f32 {
        if self.is_degenerate() || !value.is_finite() {
            return self.min;
        }
        let clamped = value.clamp(self.min, self.max);
        let Some(step) = self.step else {
            return clamped;
        };
        // Snapped from `min`, not from zero: a scale of 1..10 stepped by 2 must
        // offer 1, 3, 5 — the values the author declared — and not 2, 4, 6.
        let steps = ((clamped - self.min) / step).round();
        let snapped = self.min + steps * step;
        // Rounding can land past `max` when the span is not a whole multiple of
        // the step, and the answer is the step BELOW rather than `max` itself:
        // clamping to `max` would hand back a value the grid does not contain,
        // which is worse than the maximum being unreachable. `<input type=range>`
        // does the same — for `0..10` by `4` the highest value is 8.
        if snapped > self.max {
            (snapped - step).max(self.min)
        } else {
            snapped.max(self.min)
        }
    }

    #[doc = include_str!("slider.doc/fraction_of.md")]
    #[must_use]
    pub fn fraction_of(&self, value: f32) -> f32 {
        if self.is_degenerate() {
            return 0.0;
        }
        ((value.clamp(self.min, self.max) - self.min) / self.span()).clamp(0.0, 1.0)
    }

    #[doc = include_str!("slider.doc/value_at.md")]
    #[must_use]
    pub fn value_at(&self, fraction: f32) -> f32 {
        if self.is_degenerate() || !fraction.is_finite() {
            return self.min;
        }
        self.quantize(self.min + fraction.clamp(0.0, 1.0) * self.span())
    }

    #[doc = include_str!("slider.doc/read.md")]
    #[must_use]
    pub fn read(props: &[ViewProperty]) -> Self {
        let default = Self::default();
        Self {
            min: number(props, SLIDER_MIN).unwrap_or(default.min),
            max: number(props, SLIDER_MAX).unwrap_or(default.max),
            step: number(props, SLIDER_STEP).filter(|s| s.is_finite() && *s > 0.0),
        }
    }
}

#[doc = include_str!("slider.doc/number.md")]
pub(crate) fn number(props: &[ViewProperty], name: InternedSymbol) -> Option<f32> {
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

#[cfg(test)]
mod tests {
    use super::{SLIDER_MAX, SLIDER_MIN, SLIDER_STEP, SliderScale};
    use crate::view::{ViewProperty, ViewValue};

    #[test]
    fn a_scale_with_nothing_said_is_a_plain_fraction() {
        let scale = SliderScale::default();
        assert_eq!(scale.fraction_of(0.25), 0.25);
        assert_eq!(scale.value_at(0.25), 0.25);
    }

    #[test]
    fn a_free_scale_maps_both_ways_without_snapping() {
        let scale = SliderScale::new(10.0, 20.0);
        assert!((scale.fraction_of(15.0) - 0.5).abs() < 0.001);
        assert!((scale.value_at(0.5) - 15.0).abs() < 0.001);
        assert!((scale.value_at(0.33) - 13.3).abs() < 0.01, "no snapping");
    }

    #[test]
    fn a_step_snaps_from_the_minimum_and_not_from_zero() {
        // The rule that is easy to get wrong and visible the moment it is: a
        // scale of 1..10 by 2 offers the values the author declared.
        let scale = SliderScale::new(1.0, 10.0).stepped_by(2.0);
        assert_eq!(scale.quantize(1.4), 1.0);
        assert_eq!(scale.quantize(2.2), 3.0);
        assert_eq!(scale.quantize(4.0), 5.0);
    }

    #[test]
    fn a_step_never_offers_a_value_the_grid_does_not_contain() {
        // The surprising rule, and the right one: when the span is not a whole
        // multiple of the step, the MAXIMUM IS UNREACHABLE. Dragging to the far
        // right of a 0..10 slider stepped by 3 gives 9, because 10 is not one of
        // the values this control has. `<input type=range>` behaves identically.
        let by_three = SliderScale::new(0.0, 10.0).stepped_by(3.0);
        assert_eq!(by_three.quantize(10.0), 9.0);
        assert_eq!(by_three.quantize(9.4), 9.0);

        // And the case a naive clamp gets wrong: rounding 10 on a step of 4
        // lands on 12, and clamping THAT to `max` would answer 10 — a value the
        // grid (0, 4, 8) does not contain. The answer is the step below.
        let by_four = SliderScale::new(0.0, 10.0).stepped_by(4.0);
        assert_eq!(by_four.quantize(10.0), 8.0);
        assert_eq!(by_four.quantize(9.0), 8.0);
    }

    #[test]
    fn a_value_outside_the_bounds_is_clamped_rather_than_refused() {
        let scale = SliderScale::new(0.0, 100.0);
        assert_eq!(scale.quantize(-5.0), 0.0);
        assert_eq!(scale.quantize(500.0), 100.0);
    }

    #[test]
    fn a_non_positive_step_is_no_step_rather_than_a_division_by_it() {
        for bad in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(SliderScale::new(0.0, 1.0).stepped_by(bad).step, None);
        }
    }

    #[test]
    fn a_degenerate_scale_answers_its_minimum_rather_than_dividing_by_nothing() {
        // `min == max` is a real thing an application renders while its data
        // loads, and it must not produce a NaN that then paints nowhere.
        let flat = SliderScale::new(5.0, 5.0);
        assert!(flat.is_degenerate());
        assert_eq!(flat.fraction_of(5.0), 0.0);
        assert_eq!(flat.value_at(0.5), 5.0);
        assert_eq!(flat.quantize(7.0), 5.0);

        let inverted = SliderScale::new(10.0, 0.0);
        assert!(inverted.is_degenerate(), "max below min is not a scale");
    }

    #[test]
    fn a_non_finite_input_answers_the_minimum_rather_than_propagating() {
        let scale = SliderScale::new(0.0, 10.0);
        assert_eq!(scale.quantize(f32::NAN), 0.0);
        assert_eq!(scale.value_at(f32::NAN), 0.0);
    }

    #[test]
    fn a_scale_is_read_off_a_node_and_falls_back_where_it_says_nothing() {
        let props = [
            ViewProperty::new(SLIDER_MIN, ViewValue::Float(10.0)),
            ViewProperty::new(SLIDER_MAX, ViewValue::Int(20)),
            ViewProperty::new(SLIDER_STEP, ViewValue::Float(2.5)),
        ];
        let scale = SliderScale::read(&props);
        assert_eq!(scale.min, 10.0);
        assert_eq!(scale.max, 20.0, "an integer is a number too");
        assert_eq!(scale.step, Some(2.5));

        let bare = SliderScale::read(&[]);
        assert_eq!(bare, SliderScale::default());
    }

    #[test]
    fn round_tripping_a_stepped_value_through_its_fraction_lands_back_on_it() {
        // The property the drag path depends on: a pointer at the fraction of a
        // snapped value must produce that value again, or a thumb would creep.
        let scale = SliderScale::new(0.0, 10.0).stepped_by(1.0);
        for step in 0..=10 {
            let value = step as f32;
            assert_eq!(scale.value_at(scale.fraction_of(value)), value);
        }
    }
}
