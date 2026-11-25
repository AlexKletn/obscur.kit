//! Binds a timeline to the computed-style property it animates:
//! [`AnimationBinding`] and its [`AnimatedKind`].

use obscur_identity::NodeId;
use obscur_motion::timeline::{FillMode, TimelineId};
use obscur_motion::track::KeyframeTrack;
use obscur_style::property::PropertyId;

use super::AnimatedKind;

#[doc = include_str!("../../docs/api/orchestration/animation_binding.doc/AnimationBinding.md")]
#[derive(Debug, Clone)]
pub struct AnimationBinding {
    /// The timeline (in the scheduler) whose progress drives this animation.
    timeline: TimelineId,
    /// The node whose computed style is overridden.
    node: NodeId,
    /// The property overridden on that node.
    property: PropertyId,
    /// The keyframe track sampled by the timeline's progress.
    track: KeyframeTrack,
    /// How a sampled scalar becomes a [`PropertyValue`].
    kind: AnimatedKind,
    /// Whether the value applies during the timeline's start delay
    /// (css-animations-1 §3.9 `animation-fill-mode`).
    fill: FillMode,
}

impl AnimationBinding {
    /// Creates a binding of `timeline` to `property` on `node`, sampling `track`
    /// and lowering samples per `kind`.
    #[must_use]
    pub fn new(
        timeline: TimelineId,
        node: NodeId,
        property: PropertyId,
        track: KeyframeTrack,
        kind: AnimatedKind,
    ) -> Self {
        Self {
            timeline,
            node,
            property,
            track,
            kind,
            fill: FillMode::None,
        }
    }

    /// Sets the fill mode, returning the updated binding.
    #[must_use]
    pub fn with_fill(mut self, fill: FillMode) -> Self {
        self.fill = fill;
        self
    }

    /// Returns the fill mode.
    #[must_use]
    pub fn fill(&self) -> FillMode {
        self.fill
    }

    /// Returns the driving timeline's id.
    #[must_use]
    pub fn timeline(&self) -> TimelineId {
        self.timeline
    }

    /// Returns the animated node.
    #[must_use]
    pub fn node(&self) -> NodeId {
        self.node
    }

    /// Returns the animated property.
    #[must_use]
    pub fn property(&self) -> PropertyId {
        self.property
    }

    /// Returns the keyframe track this binding samples.
    #[must_use]
    pub fn track(&self) -> &KeyframeTrack {
        &self.track
    }

    /// Returns how sampled scalars are lowered into property values.
    #[must_use]
    pub fn kind(&self) -> AnimatedKind {
        self.kind
    }
}
