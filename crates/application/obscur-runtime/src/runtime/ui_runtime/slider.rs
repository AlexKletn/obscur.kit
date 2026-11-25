#![doc = include_str!("../../../docs/api/runtime/ui_runtime/slider.doc/_module.md")]

use obscur_geometry::Point;
use obscur_identity::NodeId;
use obscur_interaction::command::{LogicalKey, NamedKey};
use obscur_interaction::pointer::PointerId;
use obscur_view::view::EVENT_INPUT;

use crate::frame::FrameReason;
use crate::runtime::{CommandDispatch, Grab, SliderControl};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/slider.doc/begin_slider_drag.md")]
    pub(super) fn begin_slider_drag(
        &mut self,
        pointer: PointerId,
        target: NodeId,
        at: Point,
    ) -> bool {
        let Some(control) = SliderControl::enclosing(&self.tree, &self.boxes, target) else {
            return false;
        };
        let grab = control.grab_for(at);
        // Captured immediately, so a drag that leaves the track keeps steering
        // the thumb — which is what every slider anywhere does, and what a user
        // relies on when dragging fast.
        self.capture_pointer(pointer, control.node);
        self.slider_drags.insert(pointer, (control.node, grab));
        // A press JUMPS the thumb to where it landed. That is the behaviour a
        // press on the track has to have, and on a press that lands on the thumb
        // itself it changes nothing.
        self.report_slider(&control, control.moved_to(grab, at))
    }

    /// Steers the grabbed end to the pointer, or does nothing if this pointer is not dragging a slider.
    pub(super) fn continue_slider_drag(&mut self, pointer: PointerId, at: Point) -> bool {
        let Some((node, grab)) = self.slider_drags.get(&pointer).copied() else {
            return false;
        };
        let Some(control) = SliderControl::enclosing(&self.tree, &self.boxes, node) else {
            // The slider left the tree mid-drag. Forget the drag rather than
            // steering a node that is gone.
            self.slider_drags.remove(&pointer);
            return false;
        };
        self.report_slider(&control, control.moved_to(grab, at))
    }

    /// Forgets a drag, on release or cancel.
    pub(super) fn end_slider_drag(&mut self, pointer: PointerId) {
        self.slider_drags.remove(&pointer);
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/slider.doc/nudge_focused_slider.md")]
    pub(super) fn nudge_focused_slider(&mut self, key: LogicalKey) -> bool {
        let Some(focused) = self.focused else {
            return false;
        };
        let Some(control) = SliderControl::enclosing(&self.tree, &self.boxes, focused) else {
            return false;
        };
        // A keyboard moves the LOW end of a range. Which end a range's arrows
        // should drive has no obvious answer — the pointer has one because it
        // knows where it is, and the keyboard does not — so it drives the end a
        // single slider would, and a control needing both gives its thumbs
        // separate focus.
        let grab = Grab::Low;
        let ends = match key {
            LogicalKey::Named(NamedKey::ArrowRight | NamedKey::ArrowUp) => {
                control.nudged(grab, 1.0)
            }
            LogicalKey::Named(NamedKey::ArrowLeft | NamedKey::ArrowDown) => {
                control.nudged(grab, -1.0)
            }
            // Through the same door every other move uses, so the bounds obey
            // the no-crossing rule and a single slider's two ends stay one
            // point. Writing the bound straight into one end reported a RANGE
            // from a slider that has none.
            LogicalKey::Named(NamedKey::Home) => control.set(grab, control.scale.min),
            LogicalKey::Named(NamedKey::End) => control.set(grab, control.scale.max),
            _ => return false,
        };
        self.report_slider(&control, ends)
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/slider.doc/report_slider.md")]
    fn report_slider(&mut self, control: &SliderControl, ends: (f32, f32)) -> bool {
        let (low, high) = ends;
        // Nothing to report when the control did not move: a drag along a
        // stepped track spends most of its samples inside one step, and an
        // application re-rendering on every one of them would be paying for
        // motion nobody can see.
        if (low - control.low).abs() < f32::EPSILON && (high - control.high).abs() < f32::EPSILON {
            return false;
        }
        let Some((_, command)) =
            CommandDispatch::binding_for(&self.tree, control.node, EVENT_INPUT)
        else {
            return true;
        };
        self.routing.dispatch_value_input(command, low, high);
        self.request_frame(FrameReason::Input);
        true
    }
}
