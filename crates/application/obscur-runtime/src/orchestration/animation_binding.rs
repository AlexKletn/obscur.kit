//! Binds a timeline to the computed-style property it animates:
//! [`AnimationBinding`] and its [`AnimatedKind`].

use obscur_identity::NodeId;
use obscur_motion::timeline::{FillMode, TimelineId};
use obscur_motion::track::KeyframeTrack;
use obscur_motion::value::MotionValue;
use obscur_style::property::{Length, PropertyId, PropertyValue, StyleColor, Transform};

/// How a sampled scalar [`MotionValue`](obscur_motion::value::MotionValue) is
/// lowered into a concrete [`PropertyValue`] for a computed-style override.
///
/// The motion engine speaks in typed [`MotionValue`]s; the target property
/// decides what that value *means* in CSS terms. `opacity` and `flex-grow` want
/// a bare [`Number`](PropertyValue::Number) sampled from a
/// [`Scalar`](MotionValue::Scalar); `width`/`height`/margins want a pixel
/// [`Length`](PropertyValue::Length), also from a scalar; `transform` wants a
/// [`Transform`](PropertyValue::Transform) affine sampled from a
/// [`Transform`](MotionValue::Transform) matrix — which interpolates by
/// decomposition, never component-wise (CSS Transforms §14). This enum is the
/// small, property-kind-driven bridge between the two.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimatedKind {
    /// A bare number (e.g. `opacity`, `flex-grow`).
    Number,
    /// A pixel length (e.g. `width`, `height`, margins, padding).
    LengthPx,
    /// A 2D `transform` affine `[a, b, c, d, e, f]` (spec §4.17 post-layout
    /// motion; CSS Transforms §14 decomposition interpolation).
    Transform,
    /// An sRGB `color`/`background-color` etc., interpolated with premultiplied
    /// alpha from a [`Color`](MotionValue::Color) (css-color-4 §13.3).
    Color,
}

impl AnimatedKind {
    /// Lowers a sampled [`MotionValue`] into the concrete [`PropertyValue`] this
    /// kind represents, or [`None`] when the sample's variant does not match the
    /// kind (a scalar for `Number`/`LengthPx`, a transform matrix for
    /// `Transform`).
    #[must_use]
    pub fn lower(self, value: MotionValue) -> Option<PropertyValue> {
        match self {
            Self::Number => value.as_scalar().map(PropertyValue::Number),
            Self::LengthPx => value
                .as_scalar()
                .map(|s| PropertyValue::Length(Length::px(s))),
            Self::Transform => value
                .as_transform()
                .map(|matrix| PropertyValue::Transform(Transform::new(matrix))),
            Self::Color => value
                .as_color()
                .map(|[r, g, b, a]| PropertyValue::Color(StyleColor::rgba(r, g, b, a))),
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
