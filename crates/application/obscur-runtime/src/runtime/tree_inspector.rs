#![doc = include_str!("tree_inspector.doc/_module.md")]

use obscur_identity::{FastMap, NodeId};
use obscur_scene::command::DisplayList;
use obscur_style::property::PropertyId;
use obscur_tree::node::NodeKind;
use obscur_tree::tree::RetainedTree;

use super::ScrollOffsets;
use crate::orchestration::{
    ComputedStyles, GeneratedContent, InspectInfo, LayoutBoxes, NodeInspection, TreeInspection,
    overlay_commands, scroll_containers, scrollbar_rects,
};

#[doc = include_str!("tree_inspector.doc/PLACEHOLDER_PART_ID_SPACE.md")]
const PLACEHOLDER_PART_ID_SPACE: u64 = 1 << 61;

#[doc = include_str!("tree_inspector.doc/SCROLLBAR_PART_ID_SPACE.md")]
const SCROLLBAR_PART_ID_SPACE: u64 = 1 << 62;

#[doc = include_str!("tree_inspector.doc/TreeInspector.md")]
pub(crate) struct TreeInspector {
    #[doc = include_str!("tree_inspector.doc/mode.md")]
    mode: bool,
}

impl TreeInspector {
    #[doc = include_str!("tree_inspector.doc/new.md")]
    pub(crate) const fn new() -> Self {
        Self { mode: false }
    }

    #[doc = include_str!("tree_inspector.doc/set_mode.md")]
    pub(crate) fn set_mode(&mut self, on: bool) -> bool {
        let changed = self.mode != on;
        self.mode = on;
        changed
    }

    #[doc = include_str!("tree_inspector.doc/mode_2.md")]
    pub(crate) const fn mode(&self) -> bool {
        self.mode
    }

    #[doc = include_str!("tree_inspector.doc/info_at.md")]
    pub(crate) fn info_at(
        tree: &RetainedTree,
        boxes: &LayoutBoxes,
        hovered: Option<NodeId>,
    ) -> Option<InspectInfo> {
        let node = hovered?;
        let bounds = boxes.get(&node)?.bounds;
        let (component, local_name, id, classes) = match tree.get(node) {
            Some(record) => {
                let component = match record.kind() {
                    NodeKind::Element { component } => Some(*component),
                    _ => None,
                };
                let attributes = record.attributes();
                (
                    component,
                    attributes.local_name().map(str::to_owned),
                    attributes.id().map(str::to_owned),
                    attributes
                        .classes()
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                )
            }
            None => (None, None, None, Vec::new()),
        };
        Some(InspectInfo {
            node,
            component,
            local_name,
            id,
            classes,
            bounds,
        })
    }

    #[doc = include_str!("tree_inspector.doc/snapshot.md")]
    pub(crate) fn snapshot(
        tree: &RetainedTree,
        computed: &ComputedStyles,
        boxes: &LayoutBoxes,
        generated_content: &FastMap<NodeId, GeneratedContent>,
        scroll: &ScrollOffsets,
    ) -> TreeInspection {
        let mut nodes = Vec::new();
        if let Some(root) = tree.root() {
            Self::collect_inspection(tree, computed, boxes, root, None, 0, &mut nodes);
        }
        // §4.16: system-generated parts are synthetic — hidden from the component
        // tree (they are never in any node's `children()`, so `collect_inspection`
        // above never reaches them) but visible in DevTools. Project the scrollbar
        // parts of every overflowing scroll container here: the SAME track/thumb
        // rectangles the scene coordinator paints (`scrollbar_rects`), attributed to
        // the container as their parent, appended after the real tree.
        Self::collect_synthetic_scrollbar_parts(tree, computed, boxes, scroll, &mut nodes);
        // §4.16: the input placeholder is likewise a synthetic part — paint-only and
        // hidden from the component tree, yet visible here as a named "placeholder"
        // node attributed to its field. Projected from the same `generated_content`
        // the compose phase paints.
        Self::collect_synthetic_placeholder_parts(boxes, generated_content, &mut nodes);
        TreeInspection { nodes }
    }

    #[doc = include_str!("tree_inspector.doc/apply_overlay.md")]
    pub(crate) fn apply_overlay(
        &self,
        boxes: &LayoutBoxes,
        hovered: Option<NodeId>,
        display_list: DisplayList,
    ) -> DisplayList {
        if !self.mode {
            return display_list;
        }
        let Some(bounds) = hovered.and_then(|node| boxes.get(&node)) else {
            return display_list;
        };
        let mut commands = display_list.commands().to_vec();
        commands.extend(overlay_commands(bounds.bounds));
        DisplayList::from_commands(commands)
    }

    #[doc = include_str!("tree_inspector.doc/collect_synthetic_placeholder_parts.md")]
    fn collect_synthetic_placeholder_parts(
        boxes: &LayoutBoxes,
        generated_content: &FastMap<NodeId, GeneratedContent>,
        out: &mut Vec<NodeInspection>,
    ) {
        for (&field, content) in generated_content {
            if content.placeholder.is_none() {
                continue;
            }
            let depth = out
                .iter()
                .find(|n| n.node == field)
                .map_or(0, |n| n.depth + 1);
            let raw = PLACEHOLDER_PART_ID_SPACE | field.raw();
            out.push(NodeInspection {
                node: NodeId::from_raw(raw),
                parent: Some(field),
                depth,
                kind: "synthetic",
                component: None,
                local_name: Some("placeholder".to_owned()),
                id: None,
                classes: Vec::new(),
                text: None,
                bounds: boxes.get(&field).map(|layout_box| layout_box.bounds),
                styles: Vec::new(),
            });
        }
    }

    #[doc = include_str!("tree_inspector.doc/collect_synthetic_scrollbar_parts.md")]
    fn collect_synthetic_scrollbar_parts(
        tree: &RetainedTree,
        computed: &ComputedStyles,
        boxes: &LayoutBoxes,
        scroll: &ScrollOffsets,
        out: &mut Vec<NodeInspection>,
    ) {
        for container in scroll_containers(tree, computed, boxes) {
            let scroll_top = scroll.clamped(container);
            let Some((track, thumb)) =
                scrollbar_rects(container.scrollport, scroll_top, container.content_height)
            else {
                continue;
            };
            let depth = out
                .iter()
                .find(|n| n.node == container.node)
                .map_or(0, |n| n.depth + 1);
            for (index, (name, rect)) in [("scrollbar-track", track), ("scrollbar-thumb", thumb)]
                .into_iter()
                .enumerate()
            {
                let raw = SCROLLBAR_PART_ID_SPACE | (container.node.raw() << 1) | (index as u64);
                out.push(NodeInspection {
                    node: NodeId::from_raw(raw),
                    parent: Some(container.node),
                    depth,
                    kind: "synthetic",
                    component: None,
                    local_name: Some(name.to_owned()),
                    id: None,
                    classes: Vec::new(),
                    text: None,
                    bounds: Some(rect),
                    styles: Vec::new(),
                });
            }
        }
    }

    #[doc = include_str!("tree_inspector.doc/collect_inspection.md")]
    fn collect_inspection(
        tree: &RetainedTree,
        computed: &ComputedStyles,
        boxes: &LayoutBoxes,
        node: NodeId,
        parent: Option<NodeId>,
        depth: u32,
        out: &mut Vec<NodeInspection>,
    ) {
        let Some(record) = tree.get(node) else {
            return;
        };
        let (kind, component, text) = match record.kind() {
            NodeKind::Element { component } => ("element", Some(*component), None),
            NodeKind::Text { content } => ("text", None, Some(content.to_string())),
            NodeKind::Root => ("group", None, None),
            NodeKind::Synthetic { .. } => ("synthetic", None, None),
        };
        let attributes = record.attributes();
        out.push(NodeInspection {
            node,
            parent,
            depth,
            kind,
            component,
            local_name: attributes.local_name().map(str::to_owned),
            id: attributes.id().map(str::to_owned),
            classes: attributes
                .classes()
                .iter()
                .map(ToString::to_string)
                .collect(),
            text,
            bounds: boxes.get(&node).map(|layout_box| layout_box.bounds),
            styles: Self::inspect_styles(computed, node),
        });
        // Clone the child ids so the immutable borrow of `record` ends before the
        // recursive calls re-borrow the tree.
        let children = record.child_nodes().to_vec();
        for child in children {
            Self::collect_inspection(tree, computed, boxes, child, Some(node), depth + 1, out);
        }
    }

    #[doc = include_str!("tree_inspector.doc/inspect_styles.md")]
    fn inspect_styles(computed: &ComputedStyles, node: NodeId) -> Vec<(String, String)> {
        const KEYS: &[PropertyId] = &[
            PropertyId::Display,
            PropertyId::Position,
            PropertyId::Width,
            PropertyId::Height,
            PropertyId::Color,
            PropertyId::BackgroundColor,
        ];
        let Some(style) = computed.get(&node) else {
            return Vec::new();
        };
        KEYS.iter()
            .filter_map(|property| {
                style
                    .get(*property)
                    .map(|value| (format!("{property:?}"), format!("{value:?}")))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn the_overlay_starts_off_and_reports_only_real_edges() {
        let mut inspector = TreeInspector::new();
        assert!(!inspector.mode());
        assert!(inspector.set_mode(true));
        assert!(inspector.mode());
        // Setting the state it is already in is not an edge, so no frame is owed.
        assert!(!inspector.set_mode(true));
        assert!(inspector.set_mode(false));
    }

    #[test]
    fn an_overlay_that_is_off_returns_the_display_list_untouched() {
        let inspector = TreeInspector::new();
        let list = DisplayList::from_commands(Vec::new());
        let boxes = LayoutBoxes::default();
        assert!(
            inspector
                .apply_overlay(&boxes, None, list)
                .commands()
                .is_empty()
        );
    }

    #[test]
    fn an_empty_tree_projects_no_nodes_at_all() {
        let snapshot = TreeInspector::snapshot(
            &RetainedTree::new(),
            &ComputedStyles::default(),
            &LayoutBoxes::default(),
            &HashMap::default(),
            &ScrollOffsets::new(),
        );
        assert!(snapshot.nodes.is_empty());
    }
}
