//! [`PointerSessions`]: what every live pointer is currently doing (spec §2.4, §6.16).

use std::collections::HashMap;

use obscur_geometry::Point;
use obscur_identity::{FastMap, InternedSymbol, NodeId};
use obscur_interaction::gesture::{Fling, LongPressRecognizer, PanRecognizer, RecognizerId};
use obscur_interaction::pointer::{PointerButtons, PointerCaptureRegistry, PointerId};
use obscur_tree::tree::RetainedTree;

/// The maximum gap, in microseconds, between two presses for them to count as a
/// multi-click (the platform double-click window; 500 ms is the common default).
pub(crate) const MULTI_CLICK_WINDOW_MICROS: u64 = 500_000;

/// The maximum distance, in logical pixels, between two presses for them to count
/// as a multi-click — a press that jumps farther starts a fresh count.
const MULTI_CLICK_RADIUS: f32 = 4.0;

#[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/TOUCH_SLOP.md")]
pub(crate) const TOUCH_SLOP: f32 = 10.0;

/// How long a direct contact may go without moving before a lift throws nothing
/// (µs). See [`PanRecognizer::velocity_at`].
const PAN_REST_MICROS: u64 = 100_000;

#[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/FLING_TAU.md")]
const FLING_TAU: f32 = 0.325;

/// The speed (px/s) below which a coasting list is simply stopped, rather than
/// creeping toward a limit it approaches but never reaches.
const FLING_REST_SPEED: f32 = 20.0;

#[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/LONG_PRESS_SECONDS.md")]
pub(crate) const LONG_PRESS_SECONDS: f32 = 0.5;

#[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/FLING_MIN_LAUNCH_SPEED.md")]
const FLING_MIN_LAUNCH_SPEED: f32 = 50.0;

use super::{
    ActiveHold, ActivePan, PointerCompletion, pan_step::PanStep, pointer_click::PointerClick,
    press::Press,
};

#[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/PointerSessions.md")]
pub(crate) struct PointerSessions {
    pub(crate) mouse_buttons: PointerButtons,
    #[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/presses.md")]
    presses: FastMap<PointerId, Press>,
    #[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/captures.md")]
    captures: PointerCaptureRegistry,
    /// The pans armed for each pointer — one per direct contact that went down
    /// inside a user-scrollable container (spec §6.16 `Pan`).
    pans: FastMap<PointerId, ActivePan>,
    /// The scroll containers still coasting from a flick (spec §6.16).
    flings: FastMap<(NodeId, usize), Fling>,
    /// The long presses being counted, one per direct contact that went down on
    /// something with an `EVENT_LONG_PRESS` binding (spec §6.16).
    holds: FastMap<PointerId, ActiveHold>,
    /// The in-flight pointer drag-selection over the focused editable, if any:
    /// `(field node, anchor byte offset)`. The anchor is the click's byte offset,
    /// the fixed end of the selection a subsequent drag extends from (spec §5.11).
    text_drag: Option<(NodeId, usize)>,
    #[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/last_clicks.md")]
    last_clicks: FastMap<PointerId, PointerClick>,
    /// The click count of the current press: `1` single (caret), `2` double
    /// (word), `3` triple (line), cycling back to `1`.
    click_count: u32,
}

impl PointerSessions {
    /// No pointer doing anything: no press, no capture, no gesture, no coast.
    pub(crate) fn new() -> Self {
        Self {
            mouse_buttons: PointerButtons::new(),
            presses: HashMap::default(),
            captures: PointerCaptureRegistry::new(),
            pans: HashMap::default(),
            flings: HashMap::default(),
            holds: HashMap::default(),
            text_drag: None,
            last_clicks: HashMap::default(),
            click_count: 1,
        }
    }

    // ---- presses and click runs ---------------------------------------------

    #[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/begin_press.md")]
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

    #[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/take_press.md")]
    pub(crate) fn take_press(&mut self, pointer: PointerId) -> Option<Press> {
        self.presses.remove(&pointer)
    }

    #[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/pressed_targets.md")]
    pub(crate) fn pressed_targets(&self) -> Vec<NodeId> {
        self.presses
            .values()
            .filter_map(|press| press.target)
            .collect()
    }

    /// The click count of the press being resolved right now.
    pub(crate) const fn click_count(&self) -> u32 {
        self.click_count
    }

    // ---- capture (spec §2.4) -------------------------------------------------

    /// Captures `pointer` to `node`.
    pub(crate) fn capture(&mut self, pointer: PointerId, node: NodeId) {
        self.captures.capture(pointer, node);
    }

    /// Releases any capture on `pointer`, reporting whether one was held.
    pub(crate) fn release_capture(&mut self, pointer: PointerId) -> bool {
        self.captures.release(pointer).is_some()
    }

    /// The node capturing `pointer`, without checking that it is still in a tree.
    pub(crate) fn capture_target(&self, pointer: PointerId) -> Option<NodeId> {
        self.captures.target(pointer)
    }

    #[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/live_capture_target.md")]
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

    #[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/arm_pan.md")]
    pub(crate) fn arm_pan(
        &mut self,
        pointer: PointerId,
        screen_point: Point,
        time_micros: u64,
        containers: [Option<NodeId>; 2],
    ) {
        for container in containers.into_iter().flatten() {
            for axis in 0..2 {
                if self
                    .flings
                    .remove(&(container, axis))
                    .is_some_and(|f| !f.is_done())
                {
                    self.presses.remove(&pointer);
                }
            }
        }
        let id = RecognizerId::from_raw(pointer.raw());
        self.pans.insert(
            pointer,
            ActivePan {
                pan: PanRecognizer::new(id, screen_point, time_micros, TOUCH_SLOP),
                containers,
                axis: None,
            },
        );
    }

    #[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/update_pan.md")]
    pub(crate) fn update_pan(
        &mut self,
        pointer: PointerId,
        screen_point: Point,
        time_micros: u64,
    ) -> Option<PanStep> {
        let active = self.pans.get_mut(&pointer)?;
        let was_panning = active.pan.has_begun();
        let (dx, dy) = active.pan.update(screen_point, time_micros)?;
        let axis = active
            .axis
            .unwrap_or_else(|| usize::from(dx.abs() <= dy.abs()));
        let Some(container) = active.containers[axis] else {
            active.pan.cancel();
            return None;
        };
        active.axis = Some(axis);

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
            scroll_delta: if axis == 0 {
                Point::new(-dx, 0.0)
            } else {
                Point::new(0.0, -dy)
            },
            press_discarded,
        })
    }

    // ---- holds (spec §6.16) --------------------------------------------------

    /// Arms a long press for `pointer`. The caller decides whether the gesture is
    /// bound at all — arming where it is not would spend half a second of frames
    /// on every tap, almost always to conclude nothing.
    pub(crate) fn arm_hold(&mut self, pointer: PointerId, hold: ActiveHold) {
        self.holds.insert(pointer, hold);
    }

    /// Builds the armed hold for a contact that went down on `target` at
    /// `origin`, counting from [`LONG_PRESS_SECONDS`].
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

    #[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/hold_moved.md")]
    pub(crate) fn hold_moved(&mut self, pointer: PointerId, screen_point: Point) {
        if let Some(hold) = self.holds.get_mut(&pointer) {
            let travel = (screen_point.x - hold.origin.x).hypot(screen_point.y - hold.origin.y);
            hold.press.moved(travel, TOUCH_SLOP);
        }
    }

    #[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/advance_holds.md")]
    pub(crate) fn advance_holds(&mut self, delta: f32) -> Vec<(PointerId, ActiveHold)> {
        if self.holds.is_empty() {
            return Vec::new();
        }
        let fired: Vec<(PointerId, ActiveHold)> = self
            .holds
            .iter_mut()
            .filter_map(|(pointer, hold)| hold.press.advance(delta).then_some((*pointer, *hold)))
            .collect();
        for (pointer, _) in &fired {
            self.pans.remove(pointer);
            self.presses.remove(pointer);
        }
        self.holds.retain(|_, hold| hold.press.is_pending());
        fired
    }

    /// Whether any hold is still being counted, so the caller keeps asking for the
    /// frames that count it down.
    pub(crate) fn holds_pending(&self) -> bool {
        !self.holds.is_empty()
    }

    // ---- flings (spec §6.16) -------------------------------------------------

    /// Stops kinetic motion on both axes of the container.
    pub(crate) fn stop_flings(&mut self, node: NodeId) {
        self.stop_horizontal_fling(node);
        self.flings.remove(&(node, 1));
    }

    pub(crate) fn stop_horizontal_fling(&mut self, node: NodeId) {
        self.flings.remove(&(node, 0));
    }

    #[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/launch_fling.md")]
    pub(crate) fn launch_fling(&mut self, active: &ActivePan, micros: u64) -> bool {
        let Some(axis) = active.axis else {
            return false;
        };
        let Some(container) = active.containers[axis] else {
            return false;
        };
        let (vx, vy) = active.pan.velocity_at(micros, PAN_REST_MICROS);
        let velocity = [vx, vy][axis];
        if velocity.abs() < FLING_MIN_LAUNCH_SPEED {
            return false;
        }
        self.flings.insert(
            (container, axis),
            Fling::new(-velocity, FLING_TAU, FLING_REST_SPEED),
        );
        true
    }

    /// Reports whether a touch pan is currently dragging a scroller.
    pub(crate) fn has_active_pan(&self) -> bool {
        self.pans.values().any(|pan| pan.pan.has_begun())
    }

    /// Reports whether any kinetic scroll is still active.
    pub(crate) fn has_flings(&self) -> bool {
        !self.flings.is_empty()
    }

    #[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/advance_flings.md")]
    pub(crate) fn advance_flings(&mut self, delta: f32) -> Vec<((NodeId, usize), f32)> {
        self.flings
            .iter_mut()
            .map(|(node, fling)| (*node, fling.advance(delta)))
            .collect()
    }

    #[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/settle_flings.md")]
    pub(crate) fn settle_flings(&mut self, stalled: &[(NodeId, usize)]) {
        for node in stalled {
            if let Some(fling) = self.flings.get_mut(node) {
                fling.stop();
            }
        }
        self.flings.retain(|_, fling| !fling.is_done());
    }

    // ---- the text drag-selection (spec §5.11) --------------------------------

    /// The in-flight drag-selection over an editable, if one is armed.
    pub(crate) const fn text_drag(&self) -> Option<(NodeId, usize)> {
        self.text_drag
    }

    /// Arms, re-anchors or clears the drag-selection.
    pub(crate) const fn set_text_drag(&mut self, drag: Option<(NodeId, usize)>) {
        self.text_drag = drag;
    }

    // ---- the end of a sequence -----------------------------------------------

    #[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/end_pointer.md")]
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
    /// Counts what is live per kind. The recognizers themselves are internal
    /// state machines with no useful rendering here.
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
        s.arm_pan(P1, Point::new(0.0, 0.0), 0, [None, Some(node(9))]);
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
        s.arm_pan(P1, Point::new(0.0, 0.0), 0, [None, Some(node(9))]);
        // Well past the slop, and quickly: a release here would throw.
        let step = s.update_pan(P1, Point::new(0.0, -200.0), 50_000);
        assert!(step.is_some(), "the drag crossed the slop");
        assert!(s.end_pointer(P1, PointerCompletion::Cancelled).is_none());
    }

    #[test]
    fn crossing_the_slop_discards_the_press_and_captures_the_pointer() {
        let mut s = PointerSessions::new();
        s.begin_press(P1, Some(node(1)), Point::new(0.0, 0.0), 0);
        s.arm_pan(P1, Point::new(0.0, 0.0), 0, [None, Some(node(9))]);
        s.set_text_drag(Some((node(1), 3)));

        let step = s
            .update_pan(P1, Point::new(0.0, -200.0), 50_000)
            .expect("the drag crossed the slop");
        assert_eq!(step.container, node(9));
        // The content follows the finger: dragging up scrolls down.
        assert!(step.scroll_delta.y > 0.0, "delta {:?}", step.scroll_delta);
        assert!(step.press_discarded);
        assert_eq!(s.capture_target(P1), Some(node(9)));
        assert_eq!(s.text_drag(), None, "an armed drag-selection is dropped");
        assert!(s.take_press(P1).is_none(), "the press was discarded");
    }

    #[test]
    fn a_horizontal_contact_does_not_take_over_vertical_scrolling() {
        let mut sessions = PointerSessions::new();
        let start = Point::new(0.0, 0.0);
        sessions.begin_press(P1, Some(node(1)), start, 0);
        sessions.arm_pan(P1, start, 0, [None, Some(node(9))]);

        assert!(
            sessions
                .update_pan(P1, Point::new(30.0, 3.0), 10_000)
                .is_none()
        );
        assert!(
            sessions
                .update_pan(P1, Point::new(35.0, 30.0), 20_000)
                .is_none()
        );
        assert_eq!(sessions.capture_target(P1), None);
        assert!(sessions.take_press(P1).is_some());
    }

    #[test]
    fn an_unarmed_pointer_consumes_no_move() {
        let mut s = PointerSessions::new();
        assert!(s.update_pan(P1, Point::new(0.0, -200.0), 50_000).is_none());
    }
}
