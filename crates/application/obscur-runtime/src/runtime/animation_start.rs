use obscur_identity::NodeId;
use obscur_motion::track::KeyframeTrack;
use obscur_style::property::PropertyId;

use crate::orchestration::{AnimatedKind, MotionPlayback};

/// A pending CSS-`@keyframes` animation start collected during
/// the motion driver’s read stage.
pub(super) struct AnimationStart {
    pub(super) node: NodeId,
    pub(super) property: PropertyId,
    pub(super) track: KeyframeTrack,
    pub(super) duration: f32,
    pub(super) kind: AnimatedKind,
    /// The `animation-*` playback controls this animation runs under.
    pub(super) playback: MotionPlayback,
}
