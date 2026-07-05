//! Resolves a grid item's `grid-area: <name>` against its container's
//! `grid-template-areas`, patching the item's [`LayoutStyle`] placement.
//!
//! Named-area placement is the one grid feature that cannot be resolved on a
//! single node in isolation: a child's `grid-area: header` names a rectangle that
//! is defined on its **container**'s `grid-template-areas`, so the area's line
//! numbers are only knowable with the tree in hand. The CSS layer therefore
//! carries the child's reference unresolved — a [`PropertyId::GridArea`] marker
//! ([`PropertyValue::GridAreaName`]) — and this pass, which runs after
//! [`layout_styles_from`](super::LayoutCoordinator::layout_styles_from) has built
//! the per-node [`LayoutStyle`] table, looks each marker up against the parent's
//! parsed [`GridAreaTemplate`] and overwrites the item's
//! [`grid_placement`](LayoutStyle::grid_placement) with the concrete line-based
//! [`GridPlacement`] the area spans (spec §4.2).
//!
//! A child with no marker is untouched, and a marker that names an area absent
//! from the container's template (or a child whose parent authored no template)
//! is left at its all-[`GridLine::Auto`] default, so it simply auto-places —
//! keeping every non-named layout byte-for-byte identical.

use obscur_layout::algorithm::{GridLine, GridPlacement};
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{GridAreaRect, GridAreaTemplate, PropertyId, PropertyValue};
use obscur_tree::tree::RetainedTree;

use super::{ComputedStyles, LayoutStyles};

/// Resolves every grid item's `grid-area: <name>` marker against its container's
/// `grid-template-areas`, patching the item's [`LayoutStyle`] placement in place.
///
/// For each node carrying a [`PropertyId::GridArea`] name, the node's **logical
/// parent** supplies the containing grid; its computed
/// [`PropertyId::GridTemplateAreas`] template is looked up for the named area, and
/// the area's line-number rectangle is written onto the item's `grid_placement`.
/// A node without a marker, without a logical parent, whose parent authored no
/// template, or that names an unknown area is left unchanged (auto placement).
///
/// This never adds or removes entries: it only overwrites the `grid_placement`
/// field of items already present in `styles`, so it is a pure refinement of the
/// table [`layout_styles_from`](super::LayoutCoordinator::layout_styles_from)
/// built. Patches are independent per node, so the (unordered) iteration is
/// deterministic in its result.
pub fn resolve_named_grid_areas(
    styles: &mut LayoutStyles,
    tree: &RetainedTree,
    computed: &ComputedStyles,
) {
    for (&node, style) in computed {
        let Some(name) = grid_area_name_of(style) else {
            continue;
        };
        let Some(parent) = tree.get(node).and_then(|n| n.logical_parent().parent()) else {
            continue;
        };
        let Some(template) = computed.get(&parent).and_then(grid_areas_of) else {
            continue;
        };
        let Some(rect) = template.area(name) else {
            // Unknown area name: leave the item at its auto-placement default.
            continue;
        };
        if let Some(layout) = styles.get_mut(&node) {
            layout.grid_placement = placement_from_area(rect);
        }
    }
}

/// Reads a node's `grid-area: <name>` marker, or [`None`] when the node authored
/// no named-area placement (or the property is not a name marker).
fn grid_area_name_of(computed: &ComputedStyle) -> Option<&str> {
    computed
        .get(PropertyId::GridArea)
        .map(ComputedValue::value)
        .and_then(PropertyValue::as_grid_area_name)
}

/// Reads a container's parsed `grid-template-areas`, or [`None`] when it authored
/// none (or the property is not a template value).
fn grid_areas_of(computed: &ComputedStyle) -> Option<&GridAreaTemplate> {
    computed
        .get(PropertyId::GridTemplateAreas)
        .map(ComputedValue::value)
        .and_then(PropertyValue::as_grid_areas)
}

/// Converts a named area's 1-based line rectangle into a concrete
/// [`GridPlacement`] (a definite `start / end` line on both axes).
fn placement_from_area(rect: GridAreaRect) -> GridPlacement {
    GridPlacement::new(
        GridLine::Line(line(rect.column_start)),
        GridLine::Line(line(rect.column_end)),
        GridLine::Line(line(rect.row_start)),
        GridLine::Line(line(rect.row_end)),
    )
}

/// Narrows a 1-based `u32` grid line into the `i32` [`GridLine::Line`] carries,
/// saturating so a pathologically large line cannot wrap.
fn line(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::{ComponentId, NodeId};
    use obscur_layout::model::LayoutStyle;
    use obscur_tree::node::{LogicalParent, NodeKind};

    /// A three-row-by-two-column template: `"a b" "main main"`.
    fn template() -> GridAreaTemplate {
        GridAreaTemplate::from_rows(&[
            vec![Some("a".into()), Some("b".into())],
            vec![Some("main".into()), Some("main".into())],
        ])
        .expect("template")
    }

    /// A grid container with one child linked under it; returns the tree and the
    /// `(parent, child)` node ids the allocator handed back.
    fn tree_with_child() -> (RetainedTree, NodeId, NodeId) {
        let mut tree = RetainedTree::new();
        let parent = tree.alloc(NodeKind::Element {
            component: ComponentId::from_raw(1),
        });
        let child = tree.alloc(NodeKind::Element {
            component: ComponentId::from_raw(2),
        });
        tree.set_root(parent);
        if let Some(node) = tree.get_mut(parent) {
            node.push_child(child);
        }
        if let Some(node) = tree.get_mut(child) {
            node.set_logical_parent(LogicalParent::of(parent));
        }
        (tree, parent, child)
    }

    fn container_style() -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::GridTemplateAreas,
            ComputedValue::from_concrete(PropertyValue::GridAreas(template())).expect("concrete"),
        );
        style
    }

    fn item_style(name: &str) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::GridArea,
            ComputedValue::from_concrete(PropertyValue::GridAreaName(name.into()))
                .expect("concrete"),
        );
        style
    }

    #[test]
    fn named_area_resolves_to_line_placement() {
        let (tree, parent, child) = tree_with_child();

        let mut computed = ComputedStyles::new();
        computed.insert(parent, container_style());
        computed.insert(child, item_style("main"));

        let mut styles = LayoutStyles::new();
        styles.insert(parent, LayoutStyle::default());
        styles.insert(child, LayoutStyle::default());

        resolve_named_grid_areas(&mut styles, &tree, &computed);

        // `main` fills row 2, columns 1-2 → column 1/3, row 2/3.
        let placement = styles.get(&child).expect("child").grid_placement;
        assert_eq!(placement.column_start(), GridLine::Line(1));
        assert_eq!(placement.column_end(), GridLine::Line(3));
        assert_eq!(placement.row_start(), GridLine::Line(2));
        assert_eq!(placement.row_end(), GridLine::Line(3));
    }

    #[test]
    fn unknown_area_name_leaves_auto_placement() {
        let (tree, parent, child) = tree_with_child();

        let mut computed = ComputedStyles::new();
        computed.insert(parent, container_style());
        computed.insert(child, item_style("nope"));

        let mut styles = LayoutStyles::new();
        styles.insert(parent, LayoutStyle::default());
        styles.insert(child, LayoutStyle::default());

        resolve_named_grid_areas(&mut styles, &tree, &computed);

        // Unknown name → untouched all-`Auto` default.
        assert_eq!(
            styles.get(&child).expect("child").grid_placement,
            GridPlacement::default()
        );
    }

    #[test]
    fn item_without_marker_is_untouched() {
        let (tree, parent, child) = tree_with_child();

        let mut computed = ComputedStyles::new();
        computed.insert(parent, container_style());
        // Child authored no grid-area.
        computed.insert(child, ComputedStyle::new());

        let mut styles = LayoutStyles::new();
        styles.insert(child, LayoutStyle::default());

        resolve_named_grid_areas(&mut styles, &tree, &computed);

        assert_eq!(
            styles.get(&child).expect("child").grid_placement,
            GridPlacement::default()
        );
    }
}
