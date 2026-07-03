//! `FlexLayout`: single-line flex layout for the `Flex` formatting context.

use obscur_geometry::{BoxConstraints, Point, Rect, Size};

use crate::model::{AlignItems, JustifyContent, LayoutBox, LayoutStyle};

use super::{ChildMeasurer, LayoutAlgorithm, LayoutChild, LayoutOutput};

/// Single-line (no-wrap) flex layout (spec §4.1, §4.2, §4.6).
///
/// Lays items out along a main axis chosen by `style.flex_direction`, resolves
/// their main sizes by distributing free space via `flex_grow` / `flex_shrink`,
/// positions them with `justify_content` along the main axis, and aligns them
/// with `align_items` along the cross axis.
///
/// # Algorithm (Phase-1 subset)
///
/// 1. The main axis is horizontal for [`Row`]/[`RowReverse`] and vertical for
///    [`Column`]/[`ColumnReverse`]. Container content extents come from
///    `constraints.max` minus border + padding (clamped to be non-negative).
/// 2. Each item's **base main size** is `flex_basis` (resolved against the
///    container main extent); else its main-axis size (`width` for a row,
///    `height` for a column); else its measured natural main size (measured with
///    loose constraints up to the container content box). Never negative.
/// 3. `free = container_main - (sum of base sizes) - gap * (count - 1)`.
/// 4. If `free > 0` and `sum(flex_grow) > 0`, each item grows by
///    `free * grow_i / sum(grow)`. If `free < 0` and the scaled shrink factor
///    `sum(flex_shrink_i * base_i) > 0`, each item shrinks proportionally to
///    `flex_shrink_i * base_i`, clamped so no final main size is negative.
///    Otherwise base sizes are kept and the slack is handled by justification.
/// 5. Cross size per item is the container cross extent when
///    `align_items == Stretch`, else the item's measured cross size (measured
///    with its final main size fixed and a loose cross axis).
/// 6. `justify_content` turns the leftover main-axis space into a leading offset
///    and an inter-item spacing (`FlexStart`, `FlexEnd`, `Center`,
///    `SpaceBetween`, `SpaceAround`, `SpaceEvenly`); when nothing is left over
///    the leading offset and spacing are zero.
/// 7. Each item is positioned along the main axis (leading + accumulated sizes +
///    gaps + justify spacing) and along the cross axis per `align_items`
///    (`FlexStart` → 0, `FlexEnd` → `container_cross - item_cross`, `Center` →
///    half, `Stretch` → 0 filling the cross axis). Reverse directions place items
///    in reversed order along the main axis.
/// 8. Main/cross coordinates map back to x/y and are offset into the content box.
///    The node border-box size is the container content size plus border and
///    padding, then clamped into `constraints`.
///
/// [`Row`]: crate::model::FlexDirection::Row
/// [`RowReverse`]: crate::model::FlexDirection::RowReverse
/// [`Column`]: crate::model::FlexDirection::Column
/// [`ColumnReverse`]: crate::model::FlexDirection::ColumnReverse
///
/// # Out of scope for Phase 1
///
/// This is deliberately a **subset**: no wrapping / multi-line flex, no `order`
/// reordering, no baseline alignment (`AlignItems::Baseline` is not modelled),
/// no `align-self` per-item cross override, and no per-item min/max size
/// clamping beyond keeping final main sizes non-negative. These belong to later
/// phases.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FlexLayout;

impl LayoutAlgorithm for FlexLayout {
    fn layout(
        &self,
        style: &LayoutStyle,
        children: &[LayoutChild],
        constraints: BoxConstraints,
        measurer: &mut dyn ChildMeasurer,
    ) -> LayoutOutput {
        let is_row = style.flex_direction.is_row();
        let is_reverse = style.flex_direction.is_reverse();
        let gap = style.gap;

        let border = style.border;
        let padding = style.padding;
        let bp_left = border.left + padding.left;
        let bp_top = border.top + padding.top;
        let bp_x = bp_left + border.right + padding.right;
        let bp_y = bp_top + border.bottom + padding.bottom;

        // Step 1: container content extents, split into main / cross.
        let content_width = (constraints.max.width - bp_x).max(0.0);
        let content_height = (constraints.max.height - bp_y).max(0.0);
        let (container_main, container_cross) = if is_row {
            (content_width, content_height)
        } else {
            (content_height, content_width)
        };

        let count = children.len();
        let gap_span = gap * (count.saturating_sub(1) as f32);

        // Step 2: base main size per item.
        let mut base_main = Vec::with_capacity(count);
        for (index, child) in children.iter().enumerate() {
            let main_dim = if is_row {
                child.style.width
            } else {
                child.style.height
            };
            let base = child
                .style
                .flex_basis
                .resolve(container_main)
                .or_else(|| main_dim.resolve(container_main))
                .unwrap_or_else(|| {
                    let measured = measurer.measure_child(
                        index,
                        BoxConstraints::loose(Size::new(content_width, content_height)),
                    );
                    if is_row {
                        measured.width
                    } else {
                        measured.height
                    }
                });
            base_main.push(base.max(0.0));
        }

        // Step 3: free space along the main axis.
        let sum_base: f32 = base_main.iter().copied().sum();
        let free = container_main - sum_base - gap_span;

        // Step 4: resolve final main sizes via grow / shrink.
        let sum_grow: f32 = children.iter().map(|c| c.style.flex_grow).sum();
        let mut final_main: Vec<f32> = base_main.clone();
        if free > 0.0 && sum_grow > 0.0 {
            for (final_size, child) in final_main.iter_mut().zip(children.iter()) {
                *final_size += free * child.style.flex_grow / sum_grow;
            }
        } else if free < 0.0 {
            let sum_weight: f32 = children
                .iter()
                .zip(base_main.iter())
                .map(|(c, &b)| c.style.flex_shrink * b)
                .sum();
            if sum_weight > 0.0 {
                for ((final_size, child), &base) in final_main
                    .iter_mut()
                    .zip(children.iter())
                    .zip(base_main.iter())
                {
                    let weight = child.style.flex_shrink * base;
                    *final_size = (*final_size + free * weight / sum_weight).max(0.0);
                }
            }
        }

        // Step 5: cross size per item.
        let mut cross_size = Vec::with_capacity(count);
        for (index, main) in final_main.iter().copied().enumerate() {
            let cross = if style.align_items == AlignItems::Stretch {
                container_cross
            } else {
                let item_constraints = if is_row {
                    BoxConstraints {
                        min: Size::new(main, 0.0),
                        max: Size::new(main, container_cross),
                    }
                } else {
                    BoxConstraints {
                        min: Size::new(0.0, main),
                        max: Size::new(container_cross, main),
                    }
                };
                let measured = measurer.measure_child(index, item_constraints);
                if is_row {
                    measured.height
                } else {
                    measured.width
                }
            };
            cross_size.push(cross);
        }

        // Step 6: justify_content → leading offset and inter-item spacing.
        let total_final: f32 = final_main.iter().copied().sum();
        let remaining = container_main - total_final - gap_span;
        let (leading, spacing) = if remaining <= 0.0 || count == 0 {
            (0.0, 0.0)
        } else {
            let n = count as f32;
            match style.justify_content {
                JustifyContent::FlexStart => (0.0, 0.0),
                JustifyContent::FlexEnd => (remaining, 0.0),
                JustifyContent::Center => (remaining / 2.0, 0.0),
                JustifyContent::SpaceBetween => {
                    if count > 1 {
                        (0.0, remaining / (n - 1.0))
                    } else {
                        (0.0, 0.0)
                    }
                }
                JustifyContent::SpaceAround => {
                    let s = remaining / n;
                    (s / 2.0, s)
                }
                JustifyContent::SpaceEvenly => {
                    let s = remaining / (n + 1.0);
                    (s, s)
                }
            }
        };

        // Steps 7 & 8: position each item and map back to (x, y).
        let mut child_boxes = vec![LayoutBox::default(); count];
        let mut cursor = leading;
        let order: Vec<usize> = if is_reverse {
            (0..count).rev().collect()
        } else {
            (0..count).collect()
        };
        for &index in &order {
            let main = final_main[index];
            let cross = cross_size[index];
            let cross_start = match style.align_items {
                AlignItems::FlexStart | AlignItems::Stretch => 0.0,
                AlignItems::FlexEnd => container_cross - cross,
                AlignItems::Center => (container_cross - cross) / 2.0,
            };
            let (origin, size) = if is_row {
                (
                    Point::new(bp_left + cursor, bp_top + cross_start),
                    Size::new(main, cross),
                )
            } else {
                (
                    Point::new(bp_left + cross_start, bp_top + cursor),
                    Size::new(cross, main),
                )
            };
            child_boxes[index] = LayoutBox::new(Rect::new(origin, size));
            cursor += main + gap + spacing;
        }

        let border_box = Size::new(content_width + bp_x, content_height + bp_y);
        let size = constraints.constrain(border_box);

        LayoutOutput::new(size, child_boxes)
    }
}

#[cfg(test)]
mod tests {
    use super::FlexLayout;
    use crate::algorithm::{FixedChildMeasurer, LayoutAlgorithm, LayoutChild};
    use crate::model::{
        AlignItems, Dimension, FlexDirection, JustifyContent, LayoutDisplay, LayoutStyle,
    };
    use obscur_geometry::{BoxConstraints, Point, Size};
    use obscur_identity::NodeId;

    /// Container style with the given direction/justify/align and gap.
    fn container(
        direction: FlexDirection,
        justify: JustifyContent,
        align: AlignItems,
        gap: f32,
    ) -> LayoutStyle {
        LayoutStyle {
            display: LayoutDisplay::Flex,
            flex_direction: direction,
            justify_content: justify,
            align_items: align,
            gap,
            ..LayoutStyle::default()
        }
    }

    /// An item with a fixed flex-basis, no growth, no shrink.
    fn fixed_basis(raw: u64, basis: f32) -> LayoutChild {
        LayoutChild::new(
            NodeId::from_raw(raw),
            LayoutStyle {
                display: LayoutDisplay::Flex,
                flex_basis: Dimension::Px(basis),
                flex_grow: 0.0,
                flex_shrink: 0.0,
                ..LayoutStyle::default()
            },
        )
    }

    fn approx(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-4, "expected {b}, got {a}");
    }

    #[test]
    fn row_justify_flex_start_places_items_left() {
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let mut measurer = FixedChildMeasurer::default();
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[1].origin(), Point::new(50.0, 0.0));
        // Stretch fills the cross axis.
        assert_eq!(b[0].size(), Size::new(50.0, 100.0));
        assert_eq!(out.size(), Size::new(200.0, 100.0));
    }

    #[test]
    fn row_justify_variants_place_items() {
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let constraints = BoxConstraints::tight(Size::new(200.0, 100.0));

        // Center: leading = remaining/2 = 50.
        let out = FlexLayout.layout(
            &container(
                FlexDirection::Row,
                JustifyContent::Center,
                AlignItems::Stretch,
                0.0,
            ),
            &children,
            constraints,
            &mut FixedChildMeasurer::default(),
        );
        approx(out.child_boxes()[0].origin().x, 50.0);
        approx(out.child_boxes()[1].origin().x, 100.0);

        // FlexEnd: leading = remaining = 100.
        let out = FlexLayout.layout(
            &container(
                FlexDirection::Row,
                JustifyContent::FlexEnd,
                AlignItems::Stretch,
                0.0,
            ),
            &children,
            constraints,
            &mut FixedChildMeasurer::default(),
        );
        approx(out.child_boxes()[0].origin().x, 100.0);
        approx(out.child_boxes()[1].origin().x, 150.0);

        // SpaceBetween: spacing = remaining/(2-1) = 100.
        let out = FlexLayout.layout(
            &container(
                FlexDirection::Row,
                JustifyContent::SpaceBetween,
                AlignItems::Stretch,
                0.0,
            ),
            &children,
            constraints,
            &mut FixedChildMeasurer::default(),
        );
        approx(out.child_boxes()[0].origin().x, 0.0);
        approx(out.child_boxes()[1].origin().x, 150.0);

        // SpaceEvenly: s = 100/3; item0 at s, item1 at 50 + 2s.
        let s = 100.0_f32 / 3.0;
        let out = FlexLayout.layout(
            &container(
                FlexDirection::Row,
                JustifyContent::SpaceEvenly,
                AlignItems::Stretch,
                0.0,
            ),
            &children,
            constraints,
            &mut FixedChildMeasurer::default(),
        );
        approx(out.child_boxes()[0].origin().x, s);
        approx(out.child_boxes()[1].origin().x, 50.0 + 2.0 * s);
    }

    #[test]
    fn flex_grow_splits_positive_free_space() {
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let mut a = fixed_basis(1, 50.0);
        a.style.flex_grow = 1.0;
        let mut b = fixed_basis(2, 50.0);
        b.style.flex_grow = 3.0;
        let children = [a, b];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut FixedChildMeasurer::default(),
        );
        // free = 100, split 1:3 → 25 and 75 added.
        let boxes = out.child_boxes();
        approx(boxes[0].size().width, 75.0);
        approx(boxes[1].size().width, 125.0);
        approx(boxes[0].origin().x, 0.0);
        approx(boxes[1].origin().x, 75.0);
    }

    #[test]
    fn flex_shrink_absorbs_overflow() {
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let mut a = fixed_basis(1, 80.0);
        a.style.flex_shrink = 1.0;
        let mut b = fixed_basis(2, 80.0);
        b.style.flex_shrink = 1.0;
        let children = [a, b];
        // container main = 100, total base = 160, overflow = 60 split evenly.
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(100.0, 50.0)),
            &mut FixedChildMeasurer::default(),
        );
        let boxes = out.child_boxes();
        approx(boxes[0].size().width, 50.0);
        approx(boxes[1].size().width, 50.0);
        approx(boxes[1].origin().x, 50.0);
    }

    #[test]
    fn align_stretch_vs_center_sets_cross() {
        let children = [fixed_basis(1, 50.0)];
        let constraints = BoxConstraints::tight(Size::new(200.0, 100.0));

        // Stretch → item cross fills the container, at cross start 0.
        let out = FlexLayout.layout(
            &container(
                FlexDirection::Row,
                JustifyContent::FlexStart,
                AlignItems::Stretch,
                0.0,
            ),
            &children,
            constraints,
            &mut FixedChildMeasurer::new(vec![Size::new(50.0, 40.0)]),
        );
        assert_eq!(out.child_boxes()[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(out.child_boxes()[0].size(), Size::new(50.0, 100.0));

        // Center → measured cross 40, centred: (100 - 40)/2 = 30.
        let out = FlexLayout.layout(
            &container(
                FlexDirection::Row,
                JustifyContent::FlexStart,
                AlignItems::Center,
                0.0,
            ),
            &children,
            constraints,
            &mut FixedChildMeasurer::new(vec![Size::new(50.0, 40.0)]),
        );
        approx(out.child_boxes()[0].origin().y, 30.0);
        approx(out.child_boxes()[0].size().height, 40.0);
    }

    #[test]
    fn column_direction_swaps_axes() {
        let style = container(
            FlexDirection::Column,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(100.0, 200.0)),
            &mut FixedChildMeasurer::default(),
        );
        let boxes = out.child_boxes();
        // Items stack vertically; cross (width) stretched to 100.
        assert_eq!(boxes[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(boxes[0].size(), Size::new(100.0, 50.0));
        assert_eq!(boxes[1].origin(), Point::new(0.0, 50.0));
        assert_eq!(boxes[1].size(), Size::new(100.0, 50.0));
    }

    #[test]
    fn gap_is_honored() {
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            10.0,
        );
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut FixedChildMeasurer::default(),
        );
        let boxes = out.child_boxes();
        approx(boxes[0].origin().x, 0.0);
        // 50 (item0) + 10 (gap) = 60.
        approx(boxes[1].origin().x, 60.0);
    }

    #[test]
    fn row_reverse_reverses_order() {
        let style = container(
            FlexDirection::RowReverse,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut FixedChildMeasurer::default(),
        );
        let boxes = out.child_boxes();
        // Last child takes the leading slot; first child follows.
        approx(boxes[1].origin().x, 0.0);
        approx(boxes[0].origin().x, 50.0);
    }
}
