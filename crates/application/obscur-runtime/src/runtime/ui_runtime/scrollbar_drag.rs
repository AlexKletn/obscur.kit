//! Pointer capture and direct horizontal scrollbar thumb manipulation.

use obscur_geometry::Point;
use obscur_identity::{FastMap, NodeId};
use obscur_interaction::pointer::{PointerEvent, PointerEventKind, PointerId};

use crate::frame::FrameReason;
use crate::orchestration::{
    ScrollContainer, effective_stacking, horizontal_scrollbar_rects, horizontal_thumb_offset,
    overflow_scrolls_x, scroll_containers,
};
use crate::runtime::{FoldInputs, HitTesting, PointerCompletion};

use super::UiRuntime;

#[derive(Clone, Copy)]
pub(super) struct ScrollbarDrag {
    pub(super) node: NodeId,
    grab: f32,
}

impl UiRuntime {
    pub(super) fn horizontal_thumb_at(
        &self,
        target: NodeId,
        screen: Point,
    ) -> Option<ScrollbarDrag> {
        let containers = scroll_containers(&self.tree, &self.computed, &self.boxes);
        let offsets = self.scrollbar_offsets(&containers);
        let mut node = Some(target);
        let mut stacking = None;
        let mut selected = None;
        for _ in 0..self.tree.len() {
            let Some(current) = node else { break };
            if let Some(container) = containers
                .iter()
                .find(|c| c.node == current && c.horizontal_range() > 0.0)
                && overflow_scrolls_x(&self.computed, current)
            {
                let point = self.scrollbar_local_point(current, screen, &offsets);
                let port = container.scrollport;
                let offset = horizontal_thumb_offset(
                    port,
                    self.scroll.offset(*container).x,
                    container.content_width,
                    container.rtl,
                );
                if let Some((track, thumb)) =
                    horizontal_scrollbar_rects(port, offset, container.content_width)
                    && track.size.width > thumb.size.width
                    && thumb.contains(point)
                    && self.hits.is_hittable(
                        &self.tree,
                        &self.computed,
                        &self.boxes,
                        &offsets,
                        current,
                        screen,
                    )
                {
                    let stacking = self.hits.presented_stacking().unwrap_or_else(|| {
                        stacking
                            .get_or_insert_with(|| effective_stacking(&self.tree, &self.computed))
                    });
                    let band = stacking
                        .paint_band
                        .get(&current)
                        .copied()
                        .unwrap_or_default();
                    let scrollbar_band = stacking
                        .context_end_band
                        .get(&current)
                        .copied()
                        .unwrap_or(band);
                    let target_band = stacking
                        .paint_band
                        .get(&target)
                        .copied()
                        .unwrap_or_default();
                    if scrollbar_band >= target_band
                        && selected
                            .as_ref()
                            .is_none_or(|(top, _)| scrollbar_band >= *top)
                    {
                        selected = Some((
                            scrollbar_band,
                            ScrollbarDrag {
                                node: current,
                                grab: point.x - thumb.origin.x,
                            },
                        ));
                    }
                }
            }
            node = self.tree.visual_parent_of(current);
        }
        selected.map(|(_, drag)| drag)
    }

    pub(super) fn begin_scrollbar_drag(&mut self, pointer: PointerId, drag: ScrollbarDrag) {
        let had_press = self
            .pointers
            .take_press(pointer)
            .and_then(|press| press.target)
            .is_some();
        self.pointers
            .end_pointer(pointer, PointerCompletion::Cancelled);
        self.default_won_touch_gesture(pointer);
        self.pointers.stop_horizontal_fling(drag.node);
        self.pointers.capture(pointer, drag.node);
        self.scrollbar_drags.insert(pointer, drag);
        if had_press {
            self.recompute_pseudo_states();
            self.request_frame(FrameReason::Input);
        }
    }

    pub(super) fn finish_scrollbar_drag(&mut self, pointer: PointerId) {
        if let Some(drag) = self.scrollbar_drags.remove(&pointer) {
            if self.pointers.capture_target(pointer) == Some(drag.node) {
                self.pointers.release_capture(pointer);
            }
            self.touch_gestures.remove(&pointer);
        }
    }

    pub(super) fn move_scrollbar_drag(
        &mut self,
        event: &PointerEvent,
        screen: Point,
        drag: ScrollbarDrag,
        prevented: bool,
    ) {
        if event.kind() == PointerEventKind::Cancel {
            self.finish_scrollbar_drag(event.pointer());
            return;
        }
        if !prevented && matches!(event.kind(), PointerEventKind::Move | PointerEventKind::Up) {
            let containers = scroll_containers(&self.tree, &self.computed, &self.boxes);
            let Some(container) = containers
                .iter()
                .find(|c| c.node == drag.node && c.horizontal_range() > 0.0)
                .filter(|c| overflow_scrolls_x(&self.computed, c.node))
            else {
                self.finish_scrollbar_drag(event.pointer());
                return;
            };
            let port = container.scrollport;
            if let Some((track, thumb)) =
                horizontal_scrollbar_rects(port, 0.0, container.content_width)
            {
                let travel = track.size.width - thumb.size.width;
                let point = self.scrollbar_local_point(
                    drag.node,
                    screen,
                    &self.scrollbar_offsets(&containers),
                );
                if travel > 0.0 && point.x.is_finite() {
                    let fraction =
                        ((point.x - track.origin.x - drag.grab) / travel).clamp(0.0, 1.0);
                    let range = container.horizontal_range();
                    let origin = if container.rtl { range } else { 0.0 };
                    let offset = fraction * range - origin;
                    if offset != self.scroll.offset(*container).x {
                        self.scroll.set_left(
                            *container,
                            offset,
                            FoldInputs {
                                tree: &self.tree,
                                boxes: &self.boxes,
                                containers: &containers,
                            },
                        );
                        self.request_frame(FrameReason::Input);
                    }
                }
            }
        }
        if event.kind() == PointerEventKind::Up {
            self.finish_scrollbar_drag(event.pointer());
        }
    }

    fn scrollbar_offsets(&self, containers: &[ScrollContainer]) -> FastMap<NodeId, Point> {
        containers
            .iter()
            .map(|c| (c.node, self.scroll.offset(*c)))
            .collect()
    }

    fn scrollbar_local_point(
        &self,
        node: NodeId,
        screen: Point,
        offsets: &FastMap<NodeId, Point>,
    ) -> Point {
        self.hits.local_point(
            node,
            HitTesting::folded_point(&self.tree, offsets, node, screen),
        )
    }
}
