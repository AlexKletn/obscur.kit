#![doc = include_str!("animation_binding.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_motion::timeline::{FillMode, TimelineId};
use obscur_motion::track::KeyframeTrack;
use obscur_motion::value::MotionValue;
use obscur_style::property::{Length, PropertyId, PropertyValue, StyleColor, Transform};

#[doc = include_str!("animation_binding.doc/AnimatedKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimatedKind {
    #[doc = include_str!("animation_binding.doc/Number.md")]
    Number,
    #[doc = include_str!("animation_binding.doc/LengthPx.md")]
    LengthPx,
    #[doc = include_str!("animation_binding.doc/Transform.md")]
    Transform,
    #[doc = include_str!("animation_binding.doc/Color.md")]
    Color,
}

impl AnimatedKind {
    #[doc = include_str!("animation_binding.doc/lower.md")]
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

#[doc = include_str!("animation_binding.doc/AnimationBinding.md")]
#[derive(Debug, Clone)]
pub struct AnimationBinding {
    #[doc = include_str!("animation_binding.doc/timeline.md")]
    timeline: TimelineId,
    #[doc = include_str!("animation_binding.doc/node.md")]
    node: NodeId,
    #[doc = include_str!("animation_binding.doc/property.md")]
    property: PropertyId,
    #[doc = include_str!("animation_binding.doc/track.md")]
    track: KeyframeTrack,
    #[doc = include_str!("animation_binding.doc/kind.md")]
    kind: AnimatedKind,
    #[doc = include_str!("animation_binding.doc/fill.md")]
    fill: FillMode,
}

impl AnimationBinding {
    #[doc = include_str!("animation_binding.doc/new.md")]
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

    #[doc = include_str!("animation_binding.doc/with_fill.md")]
    #[must_use]
    pub fn with_fill(mut self, fill: FillMode) -> Self {
        self.fill = fill;
        self
    }

    #[doc = include_str!("animation_binding.doc/fill_2.md")]
    #[must_use]
    pub fn fill(&self) -> FillMode {
        self.fill
    }

    #[doc = include_str!("animation_binding.doc/timeline_2.md")]
    #[must_use]
    pub fn timeline(&self) -> TimelineId {
        self.timeline
    }

    #[doc = include_str!("animation_binding.doc/node_2.md")]
    #[must_use]
    pub fn node(&self) -> NodeId {
        self.node
    }

    #[doc = include_str!("animation_binding.doc/property_2.md")]
    #[must_use]
    pub fn property(&self) -> PropertyId {
        self.property
    }

    #[doc = include_str!("animation_binding.doc/track_2.md")]
    #[must_use]
    pub fn track(&self) -> &KeyframeTrack {
        &self.track
    }

    #[doc = include_str!("animation_binding.doc/kind_2.md")]
    #[must_use]
    pub fn kind(&self) -> AnimatedKind {
        self.kind
    }
}
