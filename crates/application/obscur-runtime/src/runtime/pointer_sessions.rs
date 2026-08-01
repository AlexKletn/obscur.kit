#![doc = include_str!("pointer_sessions.doc/_module.md")]

use std::collections::HashMap;

use obscur_geometry::Point;
use obscur_identity::{InternedSymbol, NodeId};
use obscur_interaction::gesture::{Fling, LongPressRecognizer, PanRecognizer, RecognizerId};
use obscur_interaction::pointer::{PointerCaptureRegistry, PointerId};
use obscur_tree::tree::RetainedTree;

#[doc = include_str!("pointer_sessions.doc/MULTI_CLICK_WINDOW_MICROS.md")]
pub(crate) const MULTI_CLICK_WINDOW_MICROS: u64 = 500_000;

#[doc = include_str!("pointer_sessions.doc/MULTI_CLICK_RADIUS.md")]
const MULTI_CLICK_RADIUS: f32 = 4.0;

#[doc = include_str!("pointer_sessions.doc/TOUCH_SLOP.md")]
pub(crate) const TOUCH_SLOP: f32 = 10.0;

#[doc = include_str!("pointer_sessions.doc/PAN_REST_MICROS.md")]
const PAN_REST_MICROS: u64 = 100_000;

#[doc = include_str!("pointer_sessions.doc/FLING_TAU.md")]
const FLING_TAU: f32 = 0.325;

#[doc = include_str!("pointer_sessions.doc/FLING_REST_SPEED.md")]
const FLING_REST_SPEED: f32 = 20.0;

#[doc = include_str!("pointer_sessions.doc/LONG_PRESS_SECONDS.md")]
pub(crate) const LONG_PRESS_SECONDS: f32 = 0.5;

#[doc = include_str!("pointer_sessions.doc/FLING_MIN_LAUNCH_SPEED.md")]
const FLING_MIN_LAUNCH_SPEED: f32 = 50.0;

#[doc = include_str!("pointer_sessions.doc/PointerClick.md")]
#[derive(Debug, Clone, Copy)]
struct PointerClick {
    #[doc = include_str!("pointer_sessions.doc/target.md")]
    target: Option<NodeId>,
    #[doc = include_str!("pointer_sessions.doc/point.md")]
    point: Point,
    #[doc = include_str!("pointer_sessions.doc/time_micros.md")]
    time_micros: u64,
    #[doc = include_str!("pointer_sessions.doc/count.md")]
    count: u32,
}

#[doc = include_str!("pointer_sessions.doc/ActiveHold.md")]
#[derive(Debug, Clone, Copy)]
pub(crate) struct ActiveHold {
    pub(crate) press: LongPressRecognizer,
    pub(crate) target: NodeId,
    pub(crate) origin: Point,
    pub(crate) trusted: bool,
    #[doc = include_str!("pointer_sessions.doc/row.md")]
    pub(crate) row: Option<(InternedSymbol, usize)>,
}

#[doc = include_str!("pointer_sessions.doc/ActivePan.md")]
#[derive(Debug, Clone, Copy)]
pub(crate) struct ActivePan {
    pan: PanRecognizer,
    container: NodeId,
}

#[doc = include_str!("pointer_sessions.doc/Press.md")]
#[derive(Debug, Clone, Copy)]
pub(crate) struct Press {
    #[doc = include_str!("pointer_sessions.doc/target_2.md")]
    pub(crate) target: Option<NodeId>,
    #[doc = include_str!("pointer_sessions.doc/clicks.md")]
    pub(crate) clicks: u32,
}

#[doc = include_str!("pointer_sessions.doc/PointerCompletion.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PointerCompletion {
    #[doc = include_str!("pointer_sessions.doc/Released.md")]
    Released,
    #[doc = include_str!("pointer_sessions.doc/Cancelled.md")]
    Cancelled,
}

#[doc = include_str!("pointer_sessions.doc/PanStep.md")]
#[derive(Debug, Clone, Copy)]
pub(crate) struct PanStep {
    #[doc = include_str!("pointer_sessions.doc/container.md")]
    pub(crate) container: NodeId,
    #[doc = include_str!("pointer_sessions.doc/scroll_delta.md")]
    pub(crate) scroll_delta: f32,
    #[doc = include_str!("pointer_sessions.doc/press_discarded.md")]
    pub(crate) press_discarded: bool,
}

#[doc = include_str!("pointer_sessions.doc/PointerSessions.md")]
pub(crate) struct PointerSessions {
    #[doc = include_str!("pointer_sessions.doc/presses.md")]
    presses: HashMap<PointerId, Press>,
    #[doc = include_str!("pointer_sessions.doc/captures.md")]
    captures: PointerCaptureRegistry,
    #[doc = include_str!("pointer_sessions.doc/pans.md")]
    pans: HashMap<PointerId, ActivePan>,
    #[doc = include_str!("pointer_sessions.doc/flings.md")]
    flings: HashMap<NodeId, Fling>,
    #[doc = include_str!("pointer_sessions.doc/holds.md")]
    holds: HashMap<PointerId, ActiveHold>,
    #[doc = include_str!("pointer_sessions.doc/text_drag.md")]
    text_drag: Option<(NodeId, usize)>,
    #[doc = include_str!("pointer_sessions.doc/last_clicks.md")]
    last_clicks: HashMap<PointerId, PointerClick>,
    #[doc = include_str!("pointer_sessions.doc/click_count.md")]
    click_count: u32,
}

impl PointerSessions {
    #[doc = include_str!("pointer_sessions.doc/new.md")]
    pub(crate) fn new() -> Self {
        Self {
            presses: HashMap::new(),
            captures: PointerCaptureRegistry::new(),
            pans: HashMap::new(),
            flings: HashMap::new(),
            holds: HashMap::new(),
            text_drag: None,
            last_clicks: HashMap::new(),
            click_count: 1,
        }
    }

    // ---- presses and click runs ---------------------------------------------

    #[doc = include_str!("pointer_sessions.doc/begin_press.md")]
    pub(crate) fn begin_press(
        &mut self,
        pointer: PointerId,
        target: Option<NodeId>,
        point: Point,
        time_micros: u64,
    ) -> Option<NodeId> {
        // Flattened deliberately: "no press outstanding" and "a press on empty
        // space" are the same *styling* state (nothing is `:active`).
        let previous_target = self.presses.get(&pointer).and_then(|press| press.target);
        let count = self
            .last_clicks
            .get(&pointer)
            .copied()
            .filter(|prev| {
                prev.target == target
                    && time_micros.saturating_sub(prev.time_micros) <= MULTI_CLICK_WINDOW_MICROS
                    && (point.x - prev.point.x).hypot(point.y - prev.point.y) <= MULTI_CLICK_RADIUS
            })
            .map_or(1, |prev| (prev.count % 3) + 1);
        self.last_clicks.insert(
            pointer,
            PointerClick {
                target,
                point,
                time_micros,
                count,
            },
        );
        self.click_count = count;
        self.presses.insert(
            pointer,
            Press {
                target,
                clicks: count,
            },
        );
        previous_target
    }

    #[doc = include_str!("pointer_sessions.doc/take_press.md")]
    pub(crate) fn take_press(&mut self, pointer: PointerId) -> Option<Press> {
        self.presses.remove(&pointer)
    }

    #[doc = include_str!("pointer_sessions.doc/pressed_targets.md")]
    pub(crate) fn pressed_targets(&self) -> Vec<NodeId> {
        self.presses
            .values()
            .filter_map(|press| press.target)
            .collect()
    }

    #[doc = include_str!("pointer_sessions.doc/click_count_2.md")]
    pub(crate) const fn click_count(&self) -> u32 {
        self.click_count
    }

    // ---- capture (spec §2.4) -------------------------------------------------

    #[doc = include_str!("pointer_sessions.doc/capture.md")]
    pub(crate) fn capture(&mut self, pointer: PointerId, node: NodeId) {
        self.captures.capture(pointer, node);
    }

    #[doc = include_str!("pointer_sessions.doc/release_capture.md")]
    pub(crate) fn release_capture(&mut self, pointer: PointerId) -> bool {
        self.captures.release(pointer).is_some()
    }

    #[doc = include_str!("pointer_sessions.doc/capture_target.md")]
    pub(crate) fn capture_target(&self, pointer: PointerId) -> Option<NodeId> {
        self.captures.target(pointer)
    }

    #[doc = include_str!("pointer_sessions.doc/live_capture_target.md")]
    pub(crate) fn live_capture_target(
        &mut self,
        pointer: PointerId,
        tree: &RetainedTree,
    ) -> Option<NodeId> {
        let node = self.captures.target(pointer)?;
        if tree.get(node).is_some() {
            return Some(node);
        }
        self.captures.release(pointer);
        None
    }

    // ---- pans (spec §6.16) ---------------------------------------------------

    #[doc = include_str!("pointer_sessions.doc/arm_pan.md")]
    pub(crate) fn arm_pan(
        &mut self,
        pointer: PointerId,
        screen_point: Point,
        time_micros: u64,
        container: NodeId,
    ) {
        if self.flings.remove(&container).is_some_and(|f| !f.is_done()) {
            self.presses.remove(&pointer);
        }
        let id = RecognizerId::from_raw(pointer.raw());
        self.pans.insert(
            pointer,
            ActivePan {
                pan: PanRecognizer::new(id, screen_point, time_micros, TOUCH_SLOP),
                container,
            },
        );
    }

    #[doc = include_str!("pointer_sessions.doc/update_pan.md")]
    pub(crate) fn update_pan(
        &mut self,
        pointer: PointerId,
        screen_point: Point,
        time_micros: u64,
    ) -> Option<PanStep> {
        let active = self.pans.get_mut(&pointer)?;
        let was_panning = active.pan.has_begun();
        let (_dx, dy) = active.pan.update(screen_point, time_micros)?;
        let container = active.container;

        let mut press_discarded = false;
        if !was_panning {
            if self
                .presses
                .remove(&pointer)
                .and_then(|press| press.target)
                .is_some()
            {
                press_discarded = true;
            }
            self.text_drag = None;
            self.captures.capture(pointer, container);
        }
        Some(PanStep {
            container,
            // The content follows the finger: dragging up (negative dy) reveals
            // what is below, which is a larger `scroll_top` — the same sign
            // convention the wheel path uses.
            scroll_delta: -dy,
            press_discarded,
        })
    }

    // ---- holds (spec §6.16) --------------------------------------------------

    #[doc = include_str!("pointer_sessions.doc/arm_hold.md")]
    pub(crate) fn arm_hold(&mut self, pointer: PointerId, hold: ActiveHold) {
        self.holds.insert(pointer, hold);
    }

    #[doc = include_str!("pointer_sessions.doc/new_hold.md")]
    pub(crate) fn new_hold(
        pointer: PointerId,
        target: NodeId,
        origin: Point,
        trusted: bool,
        row: Option<(InternedSymbol, usize)>,
    ) -> ActiveHold {
        ActiveHold {
            press: LongPressRecognizer::new(
                RecognizerId::from_raw(pointer.raw()),
                LONG_PRESS_SECONDS,
            ),
            target,
            origin,
            trusted,
            row,
        }
    }

    #[doc = include_str!("pointer_sessions.doc/hold_moved.md")]
    pub(crate) fn hold_moved(&mut self, pointer: PointerId, screen_point: Point) {
        if let Some(hold) = self.holds.get_mut(&pointer) {
            let travel = (screen_point.x - hold.origin.x).hypot(screen_point.y - hold.origin.y);
            hold.press.moved(travel, TOUCH_SLOP);
        }
    }

    #[doc = include_str!("pointer_sessions.doc/advance_holds.md")]
    pub(crate) fn advance_holds(&mut self, delta: f32) -> Vec<ActiveHold> {
        if self.holds.is_empty() {
            return Vec::new();
        }
        let fired: Vec<(PointerId, ActiveHold)> = self
            .holds
            .iter_mut()
            .filter_map(|(pointer, hold)| hold.press.advance(delta).then_some((*pointer, *hold)))
            .collect();
        for (pointer, _) in &fired {
            self.presses.remove(pointer);
        }
        self.holds.retain(|_, hold| hold.press.is_pending());
        fired.into_iter().map(|(_, hold)| hold).collect()
    }

    #[doc = include_str!("pointer_sessions.doc/holds_pending.md")]
    pub(crate) fn holds_pending(&self) -> bool {
        !self.holds.is_empty()
    }

    // ---- flings (spec §6.16) -------------------------------------------------

    #[doc = include_str!("pointer_sessions.doc/launch_fling.md")]
    pub(crate) fn launch_fling(&mut self, active: &ActivePan, micros: u64) -> bool {
        let (_, vy) = active.pan.velocity_at(micros, PAN_REST_MICROS);
        if vy.abs() < FLING_MIN_LAUNCH_SPEED {
            return false;
        }
        self.flings.insert(
            active.container,
            Fling::new(-vy, FLING_TAU, FLING_REST_SPEED),
        );
        true
    }

    #[doc = include_str!("pointer_sessions.doc/advance_flings.md")]
    pub(crate) fn advance_flings(&mut self, delta: f32) -> Vec<(NodeId, f32)> {
        self.flings
            .iter_mut()
            .map(|(node, fling)| (*node, fling.advance(delta)))
            .collect()
    }

    #[doc = include_str!("pointer_sessions.doc/settle_flings.md")]
    pub(crate) fn settle_flings(&mut self, stalled: &[NodeId]) {
        for node in stalled {
            if let Some(fling) = self.flings.get_mut(node) {
                fling.stop();
            }
        }
        self.flings.retain(|_, fling| !fling.is_done());
    }

    // ---- the text drag-selection (spec §5.11) --------------------------------

    #[doc = include_str!("pointer_sessions.doc/text_drag_2.md")]
    pub(crate) const fn text_drag(&self) -> Option<(NodeId, usize)> {
        self.text_drag
    }

    #[doc = include_str!("pointer_sessions.doc/set_text_drag.md")]
    pub(crate) const fn set_text_drag(&mut self, drag: Option<(NodeId, usize)>) {
        self.text_drag = drag;
    }

    // ---- the end of a sequence -----------------------------------------------

    #[doc = include_str!("pointer_sessions.doc/end_pointer.md")]
    pub(crate) fn end_pointer(
        &mut self,
        pointer: PointerId,
        completion: PointerCompletion,
    ) -> Option<ActivePan> {
        if let Some(hold) = self.holds.get_mut(&pointer) {
            match completion {
                PointerCompletion::Released => hold.press.end(),
                PointerCompletion::Cancelled => hold.press.cancel(),
            }
            self.holds.remove(&pointer);
        }
        let mut thrown = None;
        if let Some(mut active) = self.pans.remove(&pointer) {
            match completion {
                PointerCompletion::Released => {
                    let dragging = active.pan.has_begun();
                    active.pan.end();
                    if dragging {
                        thrown = Some(active);
                    }
                }
                PointerCompletion::Cancelled => active.pan.cancel(),
            }
        }
        // §2.4: a capture is released on pointer completion. The gesture the captor
        // claimed is over, so the next one hit-tests normally.
        self.captures.release(pointer);
        // A completed pointer ends any in-flight drag-selection: the selection
        // stays wherever the last move left it.
        self.text_drag = None;
        thrown
    }
}

impl std::fmt::Debug for PointerSessions {
    #[doc = include_str!("pointer_sessions.doc/fmt.md")]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PointerSessions")
            .field("presses", &self.presses.len())
            .field("pans", &self.pans.len())
            .field("holds", &self.holds.len())
            .field("flings", &self.flings.len())
            .field("text_drag", &self.text_drag.is_some())
            .field("click_count", &self.click_count)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::{PointerCompletion, PointerSessions};
    use obscur_geometry::Point;
    use obscur_identity::NodeId;
    use obscur_interaction::pointer::PointerId;

    const P1: PointerId = PointerId::MOUSE;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn a_click_run_chains_only_within_the_window_and_the_radius() {
        let mut s = PointerSessions::new();
        let target = Some(node(1));
        let at = Point::new(10.0, 10.0);
        let mut clicks = |time, at| {
            s.begin_press(P1, target, at, time);
            s.click_count()
        };
        assert_eq!(clicks(0, at), 1);
        assert_eq!(clicks(100_000, at), 2);
        assert_eq!(clicks(200_000, at), 3);
        // A run cycles rather than growing without bound.
        assert_eq!(clicks(300_000, at), 1);
        // Too slow: a fresh run.
        assert_eq!(clicks(1_000_000, at), 1);
        // Too far: a fresh run, even though it is in time.
        assert_eq!(clicks(1_050_000, Point::new(40.0, 10.0)), 1);
    }

    #[test]
    fn a_run_belongs_to_the_pointer_making_it() {
        let mut s = PointerSessions::new();
        let other = PointerId::from_raw(7);
        let target = Some(node(1));
        let at = Point::new(0.0, 0.0);
        s.begin_press(P1, target, at, 0);
        assert_eq!(s.click_count(), 1);
        // A second finger tapping in between is its own first click...
        s.begin_press(other, target, at, 10_000);
        assert_eq!(s.click_count(), 1);
        // ...and it does not consume the first finger's run.
        s.begin_press(P1, target, at, 20_000);
        assert_eq!(s.click_count(), 2);
    }

    #[test]
    fn beginning_a_press_reports_what_it_replaced() {
        let mut s = PointerSessions::new();
        assert_eq!(
            s.begin_press(P1, Some(node(1)), Point::new(0.0, 0.0), 0),
            None
        );
        // The outstanding press is now node 1; pressing again reports it.
        let replaced = s.begin_press(P1, Some(node(2)), Point::new(50.0, 0.0), 10_000);
        assert_eq!(replaced, Some(node(1)));
    }

    #[test]
    fn ending_a_pointer_tears_down_every_part_of_its_sequence() {
        let mut s = PointerSessions::new();
        s.begin_press(P1, Some(node(1)), Point::new(0.0, 0.0), 0);
        s.arm_pan(P1, Point::new(0.0, 0.0), 0, node(9));
        s.arm_hold(
            P1,
            PointerSessions::new_hold(P1, node(1), Point::new(0.0, 0.0), true, None),
        );
        s.capture(P1, node(9));
        s.set_text_drag(Some((node(1), 3)));
        assert!(s.holds_pending());

        // One call, and nothing of the sequence is left behind.
        assert!(s.end_pointer(P1, PointerCompletion::Cancelled).is_none());
        assert!(!s.holds_pending());
        assert_eq!(s.capture_target(P1), None);
        assert_eq!(s.text_drag(), None);
        // The pan is gone too: a second move finds nothing to feed.
        assert!(s.update_pan(P1, Point::new(0.0, 100.0), 100_000).is_none());
    }

    #[test]
    fn a_cancelled_pointer_throws_nothing_however_fast_it_was_moving() {
        let mut s = PointerSessions::new();
        s.arm_pan(P1, Point::new(0.0, 0.0), 0, node(9));
        // Well past the slop, and quickly: a release here would throw.
        let step = s.update_pan(P1, Point::new(0.0, -200.0), 50_000);
        assert!(step.is_some(), "the drag crossed the slop");
        assert!(s.end_pointer(P1, PointerCompletion::Cancelled).is_none());
    }

    #[test]
    fn crossing_the_slop_discards_the_press_and_captures_the_pointer() {
        let mut s = PointerSessions::new();
        s.begin_press(P1, Some(node(1)), Point::new(0.0, 0.0), 0);
        s.arm_pan(P1, Point::new(0.0, 0.0), 0, node(9));
        s.set_text_drag(Some((node(1), 3)));

        let step = s
            .update_pan(P1, Point::new(0.0, -200.0), 50_000)
            .expect("the drag crossed the slop");
        assert_eq!(step.container, node(9));
        // The content follows the finger: dragging up scrolls down.
        assert!(step.scroll_delta > 0.0, "delta {}", step.scroll_delta);
        assert!(step.press_discarded);
        assert_eq!(s.capture_target(P1), Some(node(9)));
        assert_eq!(s.text_drag(), None, "an armed drag-selection is dropped");
        assert!(s.take_press(P1).is_none(), "the press was discarded");
    }

    #[test]
    fn an_unarmed_pointer_consumes_no_move() {
        let mut s = PointerSessions::new();
        assert!(s.update_pan(P1, Point::new(0.0, -200.0), 50_000).is_none());
    }
}
