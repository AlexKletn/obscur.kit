#![doc = include_str!("span_attribution.doc/_module.md")]

use obscur_identity::{FrameId, NodeId, ResourceId, WindowId};

#[doc = include_str!("span_attribution.doc/SpanAttribution.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SpanAttribution {
    #[doc = include_str!("span_attribution.doc/frame.md")]
    pub frame: Option<FrameId>,
    #[doc = include_str!("span_attribution.doc/window.md")]
    pub window: Option<WindowId>,
    #[doc = include_str!("span_attribution.doc/node.md")]
    pub node: Option<NodeId>,
    #[doc = include_str!("span_attribution.doc/resource.md")]
    pub resource: Option<ResourceId>,
}

impl SpanAttribution {
    #[doc = include_str!("span_attribution.doc/none.md")]
    #[must_use]
    pub const fn none() -> Self {
        Self {
            frame: None,
            window: None,
            node: None,
            resource: None,
        }
    }

    #[doc = include_str!("span_attribution.doc/frame_2.md")]
    #[must_use]
    pub const fn frame(frame: FrameId) -> Self {
        Self {
            frame: Some(frame),
            ..Self::none()
        }
    }

    #[doc = include_str!("span_attribution.doc/with_node.md")]
    #[must_use]
    pub const fn with_node(mut self, node: NodeId) -> Self {
        self.node = Some(node);
        self
    }

    #[doc = include_str!("span_attribution.doc/with_resource.md")]
    #[must_use]
    pub const fn with_resource(mut self, resource: ResourceId) -> Self {
        self.resource = Some(resource);
        self
    }

    #[doc = include_str!("span_attribution.doc/with_window.md")]
    #[must_use]
    pub const fn with_window(mut self, window: WindowId) -> Self {
        self.window = Some(window);
        self
    }

    #[doc = include_str!("span_attribution.doc/is_empty.md")]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.frame.is_none()
            && self.window.is_none()
            && self.node.is_none()
            && self.resource.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::SpanAttribution;
    use obscur_identity::{FrameId, NodeId, ResourceId};

    #[test]
    fn an_unattributed_span_claims_nothing() {
        let attribution = SpanAttribution::none();
        assert!(attribution.is_empty());
        assert_eq!(attribution.frame, None);
    }

    #[test]
    fn a_frame_phase_span_states_its_frame_and_nothing_narrower() {
        // The shape that matters: a layout phase is about a FRAME, not a node.
        // Leaving `node` empty is the honest answer, not a gap.
        let attribution = SpanAttribution::frame(FrameId::from_raw(7));
        assert_eq!(attribution.frame, Some(FrameId::from_raw(7)));
        assert_eq!(attribution.node, None);
        assert!(!attribution.is_empty());
    }

    #[test]
    fn a_narrower_span_adds_what_it_knows() {
        let attribution = SpanAttribution::frame(FrameId::from_raw(1))
            .with_node(NodeId::from_raw(9))
            .with_resource(ResourceId::from_raw(3));
        assert_eq!(attribution.frame, Some(FrameId::from_raw(1)));
        assert_eq!(attribution.node, Some(NodeId::from_raw(9)));
        assert_eq!(attribution.resource, Some(ResourceId::from_raw(3)));
    }
}
