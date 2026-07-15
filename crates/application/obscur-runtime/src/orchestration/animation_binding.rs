//! Binds a timeline to the computed-style property it animates:
//! [`AnimationBinding`] and its [`AnimatedKind`].

use obscur_identity::NodeId;
use obscur_motion::timeline::TimelineId;
use obscur_motion::track::KeyframeTrack;
use obscur_style::property::{Length, PropertyId, PropertyValue};

/// How a sampled scalar [`MotionValue`](obscur_motion::value::MotionValue) is
/// lowered into a concrete [`PropertyValue`] for a computed-style override.
///
/// The motion engine speaks in dimensionless scalars; the target property
/// decides what that scalar *means* in CSS terms. `opacity` and `flex-grow` want
/// a bare [`Number`](PropertyValue::Number); `width`/`height`/margins want a
/// pixel [`Length`](PropertyValue::Length). This enum is the small,
/// property-kind-driven bridge between the two.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimatedKind {
    /// A bare number (e.g. `opacity`, `flex-grow`).
    Number,
    /// A pixel length (e.g. `width`, `height`, margins, padding).
    LengthPx,
}

impl AnimatedKind {
    /// Lowers a sampled `scalar` into the concrete [`PropertyValue`] this kind
    /// represents.
    #[must_use]
    pub fn to_property_value(self, scalar: f32) -> PropertyValue {
        match self {
            Self::Number => PropertyValue::Number(scalar),
            Self::LengthPx => PropertyValue::Length(Length::px(scalar)),
        }
    }
}

/// One active animation: the timeline driving it, the node and property it
/// targets, the keyframe track it samples, and how to lower the sample into a
/// computed value (spec §4.17 style → motion → paint).
///
/// The [`UiRuntime`](crate::UiRuntime) owns the
/// [`MotionScheduler`](obscur_motion::scheduler::MotionScheduler) that advances
/// the timeline; the binding is the registry entry that tells the runtime *what*
/// the timeline animates, so each frame's overlay pass can sample the track at
/// the timeline's progress and write the result onto the target node's computed
/// style before layout and paint.
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
        }
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
