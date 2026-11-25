#![doc = include_str!("../../docs/api/orchestration/formatting_context.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_layout::algorithm::{FlexLayout, FlowLayout, GridLayout, LayoutAlgorithm, LayoutChild};
use obscur_layout::model::{LayoutDisplay, LayoutStyle};
use obscur_tree::tree::RetainedTree;

use super::GridTemplates;
use super::layout_coordinator::LayoutStyles;

#[doc = include_str!("../../docs/api/orchestration/formatting_context.doc/algorithm_for.md")]
pub(super) fn algorithm_for(
    display: LayoutDisplay,
    node: NodeId,
    grid_templates: &GridTemplates,
    gap: f32,
) -> Option<Box<dyn LayoutAlgorithm>> {
    match display {
        LayoutDisplay::Block => Some(Box::new(FlowLayout)),
        LayoutDisplay::Flex => Some(Box::new(FlexLayout)),
        LayoutDisplay::Grid => Some(match grid_templates.get(&node) {
            Some(template) => Box::new(GridLayout::with_template(template.clone(), gap, gap)),
            None => Box::new(GridLayout::new(1)),
        }),
        // A `display: contents` node generates no box; [`children_of`] hoists its
        // children into the parent's formatting context, so the node itself is
        // never handed to an algorithm. Treated as box-less if ever reached.
        LayoutDisplay::Contents | LayoutDisplay::None => None,
    }
}

/// Looks up a node's [`LayoutStyle`], falling back to the default.
pub(super) fn style_of(styles: &LayoutStyles, node: NodeId) -> LayoutStyle {
    styles.get(&node).cloned().unwrap_or_default()
}

#[doc = include_str!("../../docs/api/orchestration/formatting_context.doc/children_of.md")]
pub(super) fn children_of(
    tree: &RetainedTree,
    styles: &LayoutStyles,
    node: NodeId,
) -> Vec<LayoutChild> {
    let mut out = Vec::new();
    for child in tree.visual_children(node) {
        let style = style_of(styles, child);
        // `display: contents` (css-display-3 §2.5): the element generates no box.
        // It contributes no [`LayoutChild`] of its own; instead its own children
        // are hoisted into this formatting context in its place, recursively (a
        // chain of `contents` elements flattens all the way through). Inheritance
        // and selector matching walk the untouched document tree, so only the box
        // tree is affected.
        if style.display == LayoutDisplay::Contents {
            out.extend(children_of(tree, styles, child));
        } else {
            out.push(LayoutChild::new(child, style));
        }
    }
    out
}

#[doc = include_str!("../../docs/api/orchestration/formatting_context.doc/in_flow_children.md")]
pub(super) fn in_flow_children(
    tree: &RetainedTree,
    styles: &LayoutStyles,
    node: NodeId,
) -> Vec<LayoutChild> {
    children_of(tree, styles, node)
        .into_iter()
        .filter(|child| !child.style().position.is_out_of_flow())
        .collect()
}

#[doc = include_str!("../../docs/api/orchestration/formatting_context.doc/partition_children.md")]
pub(super) fn partition_children(
    tree: &RetainedTree,
    styles: &LayoutStyles,
    node: NodeId,
) -> (Vec<LayoutChild>, Vec<LayoutChild>) {
    children_of(tree, styles, node)
        .into_iter()
        .partition(|child| !child.style().position.is_out_of_flow())
}
