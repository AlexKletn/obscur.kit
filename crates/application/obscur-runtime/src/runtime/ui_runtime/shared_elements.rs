//! Keyed layout-to-layout presentation transitions.

use obscur_geometry::{AffineTransform, Point, Rect};
use obscur_identity::NodeId;
use obscur_motion::track::{Easing, Keyframe, KeyframeTrack};
use obscur_motion::transition::{
    SharedElementKey, SharedElementTransition, TransitionInterruption,
};
use obscur_motion::value::MotionValue;
use obscur_style::property::PropertyId;
use obscur_view::view::{SHARED_ELEMENT_KEY, ViewValue};

use crate::frame::FrameReason;
use crate::orchestration::{
    AnimatedKind, ComputedStyles, scroll_containers, transform_matrix, transform_origin,
};

use super::UiRuntime;

/// A captured source rectangle and the configuration for its next destination.
pub(super) struct PendingSharedElement {
    key: SharedElementKey,
    source: Rect,
    duration: f32,
    easing: Easing,
}

impl UiRuntime {
    /// Captures the source node’s last presented geometry before a view change. The
    /// next reconciliation uses a unique destination marked with the same shared
    /// element key. Duration is in seconds. The transition changes paint geometry
    /// while layout remains at the destination. The destination and its visual
    /// ancestors may use translation and positive axis-aligned scaling.
    pub fn queue_shared_element(
        &mut self,
        source: NodeId,
        key: SharedElementKey,
        duration: f32,
        easing: Easing,
    ) -> bool {
        if key.as_str().is_empty() || !duration.is_finite() || duration <= 0.0 {
            return false;
        }
        let Some(layout) = self.boxes.get(&source) else {
            return false;
        };
        let scrolls = scroll_containers(&self.tree, &self.computed, &self.boxes);
        let Some(transform) = self.presented_local_to_window(source, &scrolls) else {
            return false;
        };
        let source = transform.map_rect(Rect::new(Point::ORIGIN, layout.size()));
        if !valid_bounds(source) {
            return false;
        }
        self.shared_elements.retain(|pending| pending.key != key);
        self.shared_elements.push(PendingSharedElement {
            key,
            source,
            duration,
            easing,
        });
        self.request_frame(FrameReason::UiCommit);
        true
    }

    /// Starts queued transitions after layout and presents the first sample before the frame is drawn.
    pub(super) fn start_shared_elements(&mut self, computed: &ComputedStyles) {
        if self.shared_elements.is_empty() {
            return;
        }
        let scrolls = scroll_containers(&self.tree, computed, &self.boxes);
        for pending in std::mem::take(&mut self.shared_elements) {
            let Some(node) = self.shared_element_destination(pending.key.as_str()) else {
                continue;
            };
            let Some(layout) = self.boxes.get(&node).copied() else {
                continue;
            };
            let style = computed.get(&node);
            let end = transform_matrix(style, layout.bounds).unwrap_or(AffineTransform::IDENTITY);
            let [end_x, end_yx, end_xy, end_y, end_dx, end_dy] = end.coeffs;
            if end_yx != 0.0 || end_xy != 0.0 || end_x <= 0.0 || end_y <= 0.0 {
                continue;
            }
            let Some(window) = self.local_to_window_with_styles(node, &scrolls, computed) else {
                continue;
            };
            let [xx, yx, xy, yy, _, _] = window.coeffs;
            if yx != 0.0 || xy != 0.0 || xx <= 0.0 || yy <= 0.0 {
                continue;
            }
            let destination = window.map_rect(Rect::new(Point::ORIGIN, layout.size()));
            if !valid_bounds(destination) || destination == pending.source {
                continue;
            }
            let transition = SharedElementTransition::new(pending.key, pending.source, destination)
                .with_easing(pending.easing);
            let origin = transform_origin(style, layout.bounds);
            let from = transition.from_bounds();
            let sx = end_x * from.size.width / destination.size.width;
            let sy = end_y * from.size.height / destination.size.height;
            let dx = end_dx
                + (from.origin.x - destination.origin.x) / (xx / end_x)
                + (sx - end_x) * (origin.x - layout.origin().x);
            let dy = end_dy
                + (from.origin.y - destination.origin.y) / (yy / end_y)
                + (sy - end_y) * (origin.y - layout.origin().y);
            let matrix = [sx, 0.0, 0.0, sy, dx, dy];
            if !matrix.iter().all(|value| value.is_finite()) {
                continue;
            }
            let initial = MotionValue::transform(matrix);
            let track = KeyframeTrack::from_keyframes(vec![
                Keyframe::new(0.0, initial).with_easing(transition.easing()),
                Keyframe::new(1.0, MotionValue::transform(end.coeffs)),
            ]);
            self.animate(
                node,
                PropertyId::Transform,
                track,
                pending.duration,
                AnimatedKind::Transform,
                TransitionInterruption::Restart,
            );
            self.motion.queue_initial_transform(node, initial);
        }
    }

    fn shared_element_destination(&self, key: &str) -> Option<NodeId> {
        let mut matches = self.tree.iter().filter_map(|(id, node)| {
            node.props()
                .iter()
                .any(|prop| {
                    prop.name() == SHARED_ELEMENT_KEY
                        && matches!(prop.value(), ViewValue::Text(name) if name.as_ref() == key)
                })
                .then_some(id)
        });
        let destination = matches.next()?;
        matches.next().is_none().then_some(destination)
    }
}

fn valid_bounds(bounds: Rect) -> bool {
    bounds.size.width > 0.0
        && bounds.size.height > 0.0
        && [
            bounds.origin.x,
            bounds.origin.y,
            bounds.right(),
            bounds.bottom(),
        ]
        .iter()
        .all(|value| value.is_finite())
}
