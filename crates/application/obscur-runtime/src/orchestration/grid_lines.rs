#![doc = include_str!("grid_lines.doc/_module.md")]

use std::collections::HashMap;

use obscur_layout::algorithm::{GridLine, GridPlacement};
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{GridLineValue, PropertyId, PropertyValue};
use obscur_tree::tree::RetainedTree;

use super::{ComputedStyles, LayoutStyles};

#[doc = include_str!("grid_lines.doc/resolve_named_grid_lines.md")]
pub fn resolve_named_grid_lines(
    styles: &mut LayoutStyles,
    tree: &RetainedTree,
    computed: &ComputedStyles,
) {
    for (&node, style) in computed {
        // Skip the common case fast: no named edge means nothing to resolve.
        if !has_named_edge(style) {
            continue;
        }
        let Some(parent) = tree.get(node).and_then(|n| n.logical_parent().parent()) else {
            continue;
        };
        let Some(parent_style) = computed.get(&parent) else {
            continue;
        };
        let Some(layout) = styles.get_mut(&node) else {
            continue;
        };

        let columns = line_name_map(parent_style, PropertyId::GridTemplateColumns);
        let rows = line_name_map(parent_style, PropertyId::GridTemplateRows);
        let placement = &layout.grid_placement;

        let column_start = resolve_edge(
            style,
            PropertyId::GridColumnStart,
            &columns,
            placement.column_start(),
        );
        let column_end = resolve_edge(
            style,
            PropertyId::GridColumnEnd,
            &columns,
            placement.column_end(),
        );
        let row_start = resolve_edge(
            style,
            PropertyId::GridRowStart,
            &rows,
            placement.row_start(),
        );
        let row_end = resolve_edge(style, PropertyId::GridRowEnd, &rows, placement.row_end());

        layout.grid_placement = GridPlacement::new(column_start, column_end, row_start, row_end);
    }
}

#[doc = include_str!("grid_lines.doc/has_named_edge.md")]
fn has_named_edge(style: &ComputedStyle) -> bool {
    [
        PropertyId::GridColumnStart,
        PropertyId::GridColumnEnd,
        PropertyId::GridRowStart,
        PropertyId::GridRowEnd,
    ]
    .into_iter()
    .any(|id| matches!(grid_line_of(style, id), Some(GridLineValue::Named(_))))
}

#[doc = include_str!("grid_lines.doc/resolve_edge.md")]
fn resolve_edge(
    style: &ComputedStyle,
    property: PropertyId,
    map: &HashMap<Box<str>, i32>,
    current: GridLine,
) -> GridLine {
    match grid_line_of(style, property) {
        Some(GridLineValue::Named(name)) => {
            map.get(name).map_or(current, |&line| GridLine::Line(line))
        }
        _ => current,
    }
}

#[doc = include_str!("grid_lines.doc/grid_line_of.md")]
fn grid_line_of(style: &ComputedStyle, property: PropertyId) -> Option<&GridLineValue> {
    style
        .get(property)
        .map(ComputedValue::value)
        .and_then(PropertyValue::as_grid_line)
}

#[doc = include_str!("grid_lines.doc/line_name_map.md")]
fn line_name_map(style: &ComputedStyle, property: PropertyId) -> HashMap<Box<str>, i32> {
    let mut map = HashMap::new();
    let Some(list) = style
        .get(property)
        .map(ComputedValue::value)
        .and_then(PropertyValue::as_grid_track_list)
    else {
        return map;
    };
    for (index, names) in list.line_names().iter().enumerate() {
        let line = i32::try_from(index).unwrap_or(i32::MAX).saturating_add(1);
        for name in names {
            map.entry(name.clone()).or_insert(line);
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::{ComponentId, NodeId};
    use obscur_layout::model::LayoutStyle;
    use obscur_style::property::{GridTrack, GridTrackList, Length};
    use obscur_tree::node::{LogicalParent, NodeKind};

    /// A grid container with one child linked under it; returns the tree and the
    /// `(parent, child)` node ids.
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

    /// A container whose columns declare lines `[a] 100px [b] 100px [c] 100px [d]`
    /// (lines a=1, b=2, c=3, d=4).
    fn container_with_named_columns() -> ComputedStyle {
        let list = GridTrackList::new(
            Box::from([
                GridTrack::Fixed(Length::px(100.0)),
                GridTrack::Fixed(Length::px(100.0)),
                GridTrack::Fixed(Length::px(100.0)),
            ]),
            Box::from([
                Box::from([Box::<str>::from("a")]) as Box<[Box<str>]>,
                Box::from([Box::<str>::from("b")]),
                Box::from([Box::<str>::from("c")]),
                Box::from([Box::<str>::from("d")]),
            ]),
        );
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::GridTemplateColumns,
            ComputedValue::from_concrete(PropertyValue::grid_track_list(list)).expect("concrete"),
        );
        style
    }

    /// A grid item authoring `grid-column: <start> / <end>` as named edges.
    fn item_with_named_columns(start: &str, end: &str) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::GridColumnStart,
            ComputedValue::from_concrete(PropertyValue::GridLine(GridLineValue::Named(
                start.into(),
            )))
            .expect("concrete"),
        );
        style.set(
            PropertyId::GridColumnEnd,
            ComputedValue::from_concrete(PropertyValue::GridLine(GridLineValue::Named(end.into())))
                .expect("concrete"),
        );
        style
    }

    #[test]
    fn named_columns_resolve_to_line_placement() {
        let (tree, parent, child) = tree_with_child();
        let mut computed = ComputedStyles::default();
        computed.insert(parent, container_with_named_columns());
        computed.insert(child, item_with_named_columns("b", "c"));

        let mut styles = LayoutStyles::default();
        styles.insert(parent, LayoutStyle::default());
        styles.insert(child, LayoutStyle::default());

        resolve_named_grid_lines(&mut styles, &tree, &computed);

        let placement = styles.get(&child).expect("child").grid_placement;
        assert_eq!(placement.column_start(), GridLine::Line(2));
        assert_eq!(placement.column_end(), GridLine::Line(3));
        // Rows were never named → untouched auto.
        assert_eq!(placement.row_start(), GridLine::Auto);
        assert_eq!(placement.row_end(), GridLine::Auto);
    }

    #[test]
    fn unknown_line_name_leaves_auto_placement() {
        let (tree, parent, child) = tree_with_child();
        let mut computed = ComputedStyles::default();
        computed.insert(parent, container_with_named_columns());
        computed.insert(child, item_with_named_columns("nope", "c"));

        let mut styles = LayoutStyles::default();
        styles.insert(child, LayoutStyle::default());

        resolve_named_grid_lines(&mut styles, &tree, &computed);

        let placement = styles.get(&child).expect("child").grid_placement;
        // Unknown start name stays auto; the known end resolves.
        assert_eq!(placement.column_start(), GridLine::Auto);
        assert_eq!(placement.column_end(), GridLine::Line(3));
    }

    #[test]
    fn item_without_named_edge_is_untouched() {
        let (tree, parent, child) = tree_with_child();
        let mut computed = ComputedStyles::default();
        computed.insert(parent, container_with_named_columns());
        computed.insert(child, ComputedStyle::new());

        let mut styles = LayoutStyles::default();
        styles.insert(child, LayoutStyle::default());

        resolve_named_grid_lines(&mut styles, &tree, &computed);

        assert_eq!(
            styles.get(&child).expect("child").grid_placement,
            GridPlacement::default()
        );
    }
}
