//! `SpanAttribution`: what a performance span is *about* (spec §7.9).

use obscur_identity::{FrameId, NodeId, ResourceId, WindowId};

/// What a [`PerformanceSpan`](crate::performance::PerformanceSpan) is attributed
/// to, beyond its subsystem (spec §7.9).
///
/// §7.9 lists frame, window, subsystem, node, layout context, resource,
/// invalidation cause and plugin contribution. Subsystem lives on the span
/// itself; the rest that this crate can *name* live here.
///
/// # Every field is optional, and that is the honest shape
///
/// A span is attributed to what its emitter actually knows. A frame-phase span
/// knows its frame and nothing narrower — there is no single node a whole layout
/// phase is "about". A per-node span would set `node`; a resource load would set
/// `resource`. Forcing a value where the emitter has none would mean inventing
/// attribution, which is worse than reporting none: a consumer cannot tell an
/// invented answer from a measured one.
///
/// # What is deliberately absent
///
/// **Layout context**, **invalidation cause** and **plugin contribution** from
/// §7.9's list are not here. Each is named by a type that lives *above* this
/// foundation crate (`InvalidationReason` is a runtime concept, a layout context
/// a layout one), so carrying them would mean either a dependency inversion or a
/// duplicated enum that silently drifts from the real one. They wait for their
/// owners to expose an identity this tier can hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SpanAttribution {
    /// The frame this span was recorded during.
    pub frame: Option<FrameId>,
    /// The window the work belongs to, for a multi-surface host.
    pub window: Option<WindowId>,
    /// The node the work is about, for a span narrow enough to have one.
    pub node: Option<NodeId>,
    /// The resource the work is about, for a load/decode span.
    pub resource: Option<ResourceId>,
}

impl SpanAttribution {
    /// Attributed to nothing — the honest default for an emitter that knows no
    /// more than its subsystem.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            frame: None,
            window: None,
            node: None,
            resource: None,
        }
    }

    /// Attributed to `frame` — what every frame-phase span can state.
    #[must_use]
    pub const fn frame(frame: FrameId) -> Self {
        Self {
            frame: Some(frame),
            ..Self::none()
        }
    }

    /// Returns this attribution narrowed to `node` (fluent).
    #[must_use]
    pub const fn with_node(mut self, node: NodeId) -> Self {
        self.node = Some(node);
        self
    }

    /// Returns this attribution narrowed to `resource` (fluent).
    #[must_use]
    pub const fn with_resource(mut self, resource: ResourceId) -> Self {
        self.resource = Some(resource);
        self
    }

    /// Returns this attribution narrowed to `window` (fluent).
    #[must_use]
    pub const fn with_window(mut self, window: WindowId) -> Self {
        self.window = Some(window);
        self
    }

    /// Whether nothing is attributed.
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
