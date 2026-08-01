#![doc = include_str!("grid_areas.doc/_module.md")]

use obscur_layout::algorithm::{GridLine, GridPlacement};
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{GridAreaRect, GridAreaTemplate, PropertyId, PropertyValue};
use obscur_tree::tree::RetainedTree;

use super::{ComputedStyles, LayoutStyles};

#[doc = include_str!("grid_areas.doc/resolve_named_grid_areas.md")]
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

#[doc = include_str!("grid_areas.doc/grid_area_name_of.md")]
fn grid_area_name_of(computed: &ComputedStyle) -> Option<&str> {
    computed
        .get(PropertyId::GridArea)
        .map(ComputedValue::value)
        .and_then(PropertyValue::as_grid_area_name)
}

#[doc = include_str!("grid_areas.doc/grid_areas_of.md")]
fn grid_areas_of(computed: &ComputedStyle) -> Option<&GridAreaTemplate> {
    computed
        .get(PropertyId::GridTemplateAreas)
        .map(ComputedValue::value)
        .and_then(PropertyValue::as_grid_areas)
}

#[doc = include_str!("grid_areas.doc/placement_from_area.md")]
fn placement_from_area(rect: GridAreaRect) -> GridPlacement {
    GridPlacement::new(
        GridLine::Line(line(rect.column_start)),
        GridLine::Line(line(rect.column_end)),
        GridLine::Line(line(rect.row_start)),
        GridLine::Line(line(rect.row_end)),
    )
}

#[doc = include_str!("grid_areas.doc/line.md")]
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
