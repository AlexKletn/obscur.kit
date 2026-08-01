#![doc = include_str!("intrinsic_inline.doc/_module.md")]

use obscur_geometry::{BoxConstraints, Size};

use crate::algorithm::{ChildMeasurer, LayoutChild};
use crate::model::Dimension;

#[doc = include_str!("intrinsic_inline.doc/UNBOUNDED.md")]
const UNBOUNDED: f32 = f32::INFINITY;

#[doc = include_str!("intrinsic_inline.doc/probe.md")]
fn probe(
    children: &[LayoutChild],
    offer: f32,
    block_space: f32,
    measurer: &mut dyn ChildMeasurer,
) -> f32 {
    let mut widest = 0.0_f32;
    for (index, child) in children.iter().enumerate() {
        // A zero basis, deliberately: this is an intrinsic contribution, and CSS
        // Sizing 3 §5.2 treats a percentage margin as zero while computing one —
        // the containing block's size is what is being computed, so there is
        // nothing definite for the percentage to be a fraction of.
        let margin = child.style().margin.resolve(0.0);
        let margins = margin.left + margin.right;
        // `UNBOUNDED - margins` stays unbounded; a zero offer stays zero.
        let available = (offer - margins).max(0.0);
        let size = measurer.measure_child(
            index,
            BoxConstraints::loose(Size::new(available, block_space)),
        );
        widest = widest.max(size.width + margins);
    }
    widest
}

#[doc = include_str!("intrinsic_inline.doc/intrinsic_inline_size.md")]
pub(crate) fn intrinsic_inline_size(
    width: Dimension,
    children: &[LayoutChild],
    stretch: f32,
    block_space: f32,
    measurer: &mut dyn ChildMeasurer,
) -> Option<f32> {
    match width {
        Dimension::MinContent => Some(probe(children, 0.0, block_space, measurer)),
        Dimension::MaxContent => Some(probe(children, UNBOUNDED, block_space, measurer)),
        Dimension::FitContent => {
            let min = probe(children, 0.0, block_space, measurer);
            let max = probe(children, UNBOUNDED, block_space, measurer);
            Some(max.min(stretch.max(min)))
        }
        Dimension::Auto | Dimension::Px(_) | Dimension::Percent(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{intrinsic_inline_size, probe};
    use crate::algorithm::{ChildMeasurer, LayoutChild};
    use crate::model::{Dimension, LayoutStyle, MarginEdges};
    use obscur_geometry::{BoxConstraints, Size};
    use obscur_identity::NodeId;

    /// A child that behaves like a paragraph: it fills the width it is offered,
    /// but never goes below `min` (its longest unbreakable word) and never above
    /// `max` (its whole content on one line).
    struct Paragraph {
        min: f32,
        max: f32,
    }

    impl ChildMeasurer for Paragraph {
        fn measure_child(&mut self, _index: usize, constraints: BoxConstraints) -> Size {
            let width = constraints.max.width.clamp(self.min, self.max);
            Size::new(width, 10.0)
        }
    }

    fn child(margin: MarginEdges) -> LayoutChild {
        LayoutChild::new(
            NodeId::from_raw(1),
            LayoutStyle {
                margin,
                ..LayoutStyle::default()
            },
        )
    }

    fn plain() -> LayoutChild {
        child(MarginEdges::ZERO)
    }

    fn one() -> Vec<LayoutChild> {
        vec![plain()]
    }

    #[test]
    fn min_content_offers_nothing_and_gets_the_unbreakable_width() {
        let mut m = Paragraph {
            min: 30.0,
            max: 200.0,
        };
        assert_eq!(probe(&one(), 0.0, 100.0, &mut m), 30.0);
    }

    #[test]
    fn max_content_offers_unbounded_space_and_gets_the_single_line_width() {
        // The probe must not hand the child infinity as its *size*: the child
        // clamps to what it actually wants, and that is what comes back.
        let mut m = Paragraph {
            min: 30.0,
            max: 200.0,
        };
        let width = probe(&one(), f32::INFINITY, 100.0, &mut m);
        assert_eq!(width, 200.0);
        assert!(
            width.is_finite(),
            "an unbounded offer must yield a finite want"
        );
    }

    #[test]
    fn a_blocks_widest_child_decides_not_the_sum() {
        // Blocks stack vertically: two 200-wide children want 200, not 400.
        struct Fixed;
        impl ChildMeasurer for Fixed {
            fn measure_child(&mut self, index: usize, _c: BoxConstraints) -> Size {
                Size::new(if index == 0 { 80.0 } else { 200.0 }, 10.0)
            }
        }
        let children = vec![plain(), plain()];
        assert_eq!(probe(&children, f32::INFINITY, 100.0, &mut Fixed), 200.0);
    }

    #[test]
    fn a_childs_inline_margins_count_toward_the_intrinsic_width() {
        // §10.3.3: the margins are part of the space the run needs.
        struct Fixed;
        impl ChildMeasurer for Fixed {
            fn measure_child(&mut self, _index: usize, _c: BoxConstraints) -> Size {
                Size::new(100.0, 10.0)
            }
        }
        let children = vec![child(MarginEdges::px(0.0, 25.0, 0.0, 15.0))];
        assert_eq!(
            probe(&children, f32::INFINITY, 100.0, &mut Fixed),
            140.0,
            "100 of content + 15 left + 25 right"
        );
    }

    #[test]
    fn fit_content_takes_the_content_when_it_fits() {
        // max-content (200) <= stretch (500) → the box shrinks to its content.
        let mut m = Paragraph {
            min: 30.0,
            max: 200.0,
        };
        let width = intrinsic_inline_size(Dimension::FitContent, &one(), 500.0, 100.0, &mut m);
        assert_eq!(width, Some(200.0));
    }

    #[test]
    fn fit_content_falls_back_to_the_space_available_when_content_is_wider() {
        // stretch (100) < max-content (200), and 100 is still above min-content
        // (30) → the box takes the space it is given.
        let mut m = Paragraph {
            min: 30.0,
            max: 200.0,
        };
        let width = intrinsic_inline_size(Dimension::FitContent, &one(), 100.0, 100.0, &mut m);
        assert_eq!(width, Some(100.0));
    }

    #[test]
    fn fit_content_never_squeezes_below_min_content() {
        // stretch (10) is narrower than min-content (30): the unbreakable word
        // wins and the box overflows, as CSS 2.1 §10.3.5 requires.
        let mut m = Paragraph {
            min: 30.0,
            max: 200.0,
        };
        let width = intrinsic_inline_size(Dimension::FitContent, &one(), 10.0, 100.0, &mut m);
        assert_eq!(width, Some(30.0));
    }

    #[test]
    fn a_non_intrinsic_width_is_not_this_functions_business() {
        let mut m = Paragraph {
            min: 30.0,
            max: 200.0,
        };
        for dim in [
            Dimension::Auto,
            Dimension::Px(50.0),
            Dimension::Percent(50.0),
        ] {
            assert_eq!(
                intrinsic_inline_size(dim, &one(), 100.0, 100.0, &mut m),
                None,
                "{dim:?} resolves by arithmetic, not by measuring"
            );
        }
    }

    #[test]
    fn min_content_and_max_content_disagree_for_wrappable_content() {
        // The whole point of having two: if these ever returned the same number
        // for a paragraph, the probes are not actually probing.
        let mut m = Paragraph {
            min: 30.0,
            max: 200.0,
        };
        let min = intrinsic_inline_size(Dimension::MinContent, &one(), 100.0, 100.0, &mut m);
        let max = intrinsic_inline_size(Dimension::MaxContent, &one(), 100.0, 100.0, &mut m);
        assert_eq!(min, Some(30.0));
        assert_eq!(max, Some(200.0));
        assert!(min < max);
    }
}
