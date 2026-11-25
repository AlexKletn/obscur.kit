use obscur_identity::NodeId;
use obscur_motion::{track::Easing, value::MotionValue};
use obscur_style::property::PropertyId;

use crate::orchestration::AnimatedKind;

use super::transition_run::TransitionRun;

/// A pending CSS-transition start collected during
/// the motion driver’s read stage.
pub(super) struct TransitionStart {
    pub(super) node: NodeId,
    pub(super) property: PropertyId,
    pub(super) from: MotionValue,
    pub(super) to: MotionValue,
    /// The **effective** duration, in seconds — the matching `transition-duration`
    /// already shortened by the reversing shortening factor when this start
    /// reverses an in-flight transition (css-transitions-1 §3.1).
    pub(super) duration: f32,
    /// The effective `transition-delay`, in seconds (negative starts part-way
    /// through); a negative delay is likewise shortened by the reversing factor.
    pub(super) delay: f32,
    pub(super) easing: Easing,
    pub(super) kind: AnimatedKind,
    /// The reversing bookkeeping to retain for this `(node, property)` once the
    /// transition starts, so a later interruption can shorten *its* reversal.
    pub(super) run: TransitionRun,
}
