#![doc = include_str!("snapshot_normalizer.doc/_module.md")]

use std::collections::BTreeSet;

use obscur_identity::NodeId;

use super::{NormalizedNode, NormalizedPipeline, PipelineSnapshot};

#[doc = include_str!("snapshot_normalizer.doc/SnapshotNormalizer.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct SnapshotNormalizer;

impl SnapshotNormalizer {
    #[doc = include_str!("snapshot_normalizer.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("snapshot_normalizer.doc/normalize.md")]
    #[must_use]
    pub fn normalize(&self, snapshot: &PipelineSnapshot) -> NormalizedPipeline {
        let computed = snapshot.computed();
        let layout = snapshot.layout();

        // Union of node ids that carry style or layout data. A `BTreeSet`
        // dedupes and yields the ids already sorted by `NodeId` (i.e. by
        // `.raw()`), which is the stable order the normalized form guarantees.
        let ids: BTreeSet<NodeId> = computed.keys().chain(layout.keys()).copied().collect();

        let nodes: Vec<NormalizedNode> = ids
            .into_iter()
            .map(|id| {
                let opacity = computed
                    .get(&id)
                    .and_then(obscur_style::computed::ComputedStyle::opacity);
                let layout_bounds = layout.get(&id).map(|layout_box| layout_box.bounds);
                NormalizedNode::new(id, opacity, layout_bounds)
            })
            .collect();

        NormalizedPipeline::new(
            snapshot.tree().root(),
            snapshot.tree().len(),
            nodes,
            snapshot.fill_count(),
            snapshot.is_balanced(),
            snapshot.paint().len(),
            snapshot.accessibility().len(),
            snapshot.accessibility().root(),
            snapshot.render_graph().pass_count(),
            snapshot.image().cloned(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;
    use obscur_style::cascade::ResolvedDeclarations;
    use obscur_style::property::{Length, PropertyId, PropertyValue, StyleColor};
    use obscur_view::view::{View, ViewFragment, ViewNode};

    use crate::host::HeadlessHost;
    use obscur_identity::RuntimeId;
    use obscur_runtime::orchestration::StyleInputs;

    /// A small view: a root fragment with two element children.
    fn sample_view() -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10)),
            ViewNode::element(ComponentId::from_raw(11)),
        ]))
    }

    /// Cascaded declarations giving a node a solid background and a fixed size,
    /// so it produces a layout box and a scene fill.
    fn styled_box_decls() -> ResolvedDeclarations {
        let mut decls = ResolvedDeclarations::new();
        decls.insert(
            PropertyId::BackgroundColor,
            PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)),
        );
        decls.insert(PropertyId::Width, PropertyValue::Length(Length::px(100.0)));
        decls.insert(PropertyId::Height, PropertyValue::Length(Length::px(50.0)));
        decls
    }

    /// Renders `sample_view` into a fresh host and returns the host so the test
    /// can read its retained tree and viewport.
    ///
    /// No stylesheet is needed: every caller styles through the `StyleInputs` it
    /// hands to `PipelineSnapshot::capture`, which resolves them itself.
    fn rendered_host() -> HeadlessHost {
        let mut host = HeadlessHost::new(RuntimeId::from_raw(1));
        host.render_styled(&sample_view(), &[]);
        host
    }

    /// Returns the first child id of the retained-tree root, if any.
    fn first_child(host: &HeadlessHost) -> Option<NodeId> {
        let tree = host.tree();
        let root = tree.root()?;
        tree.get(root)?.child_nodes().first().copied()
    }

    #[test]
    fn capture_is_deterministic_across_two_captures() {
        // §5.13 deterministic-slice guarantee: capture twice → equal, and their
        // normalized forms are equal.
        let inputs = StyleInputs::new();
        let host = rendered_host();
        let viewport = host.config().viewport();

        let first = PipelineSnapshot::capture(host.tree(), &inputs, viewport);
        let second = PipelineSnapshot::capture(host.tree(), &inputs, viewport);
        assert_eq!(first, second, "two captures of the same frame are equal");

        let normalizer = SnapshotNormalizer::new();
        assert_eq!(
            normalizer.normalize(&first),
            normalizer.normalize(&second),
            "normalized forms of equal captures are equal",
        );
    }

    #[test]
    fn styled_child_produces_content_and_sorted_nodes() {
        // Style a discovered child node so the pipeline has real content.
        let mut host = rendered_host();
        let child = first_child(&host).expect("root has at least one child");

        let mut inputs = StyleInputs::new();
        inputs.set_cascaded(child, styled_box_decls());

        host.render_styled(&sample_view(), &[]);
        let snapshot = PipelineSnapshot::capture(host.tree(), &inputs, host.config().viewport());

        assert!(snapshot.node_count() > 0, "the tree has nodes");
        assert!(snapshot.fill_count() >= 1, "the styled child emits a fill");
        assert!(snapshot.is_balanced(), "the display list is balanced");

        let normalized = SnapshotNormalizer::new().normalize(&snapshot);
        assert_eq!(normalized.node_count(), snapshot.node_count());
        assert_eq!(normalized.fill_count(), snapshot.fill_count());
        assert!(normalized.scene_balanced());

        let ids: Vec<u64> = normalized.nodes().iter().map(|n| n.node().raw()).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted, "normalized nodes are sorted by NodeId");

        // The styled child appears with its layout bounds.
        let child_row = normalized
            .nodes()
            .iter()
            .find(|n| n.node() == child)
            .expect("styled child is present in the normalized nodes");
        assert!(
            child_row.layout_bounds().is_some(),
            "the styled child was arranged",
        );
    }

    #[test]
    fn normalized_form_carries_the_new_stage_summaries() {
        // The paint, accessibility, and render-graph summaries survive
        // normalization and are deterministic across two captures.
        let inputs = StyleInputs::new();
        let host = rendered_host();
        let viewport = host.config().viewport();

        let snapshot = PipelineSnapshot::capture(host.tree(), &inputs, viewport);
        let normalized = SnapshotNormalizer::new().normalize(&snapshot);

        assert_eq!(normalized.paint_command_count(), snapshot.paint().len());
        assert_eq!(
            normalized.accessibility_node_count(),
            snapshot.accessibility().len()
        );
        assert_eq!(
            normalized.accessibility_root(),
            snapshot.accessibility().root()
        );
        assert_eq!(
            normalized.render_pass_count(),
            snapshot.render_graph().pass_count()
        );
        assert!(normalized.image().is_none(), "no backend attached an image");

        // Deterministic across captures.
        let again = SnapshotNormalizer::new().normalize(&PipelineSnapshot::capture(
            host.tree(),
            &inputs,
            viewport,
        ));
        assert_eq!(normalized, again);
    }

    #[test]
    fn empty_snapshot_normalizes_to_empty_rows() {
        let inputs = StyleInputs::new();
        // No render: the tree is empty.
        let host = HeadlessHost::new(RuntimeId::from_raw(2));
        let snapshot = PipelineSnapshot::capture(host.tree(), &inputs, host.config().viewport());
        let normalized = SnapshotNormalizer::new().normalize(&snapshot);

        assert_eq!(normalized.node_count(), 0);
        assert!(normalized.nodes().is_empty());
        assert_eq!(normalized.root(), None);
        assert_eq!(normalized.fill_count(), 0);
        assert!(normalized.scene_balanced());
    }
}
