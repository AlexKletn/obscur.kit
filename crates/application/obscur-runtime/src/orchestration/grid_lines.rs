//! Resolves a grid item's named line placement (`grid-column: <name>` /
//! `grid-row: <name>`) against its container's `grid-template-columns` /
//! `grid-template-rows` line names, patching the item's [`LayoutStyle`] placement.
//!
//! Named-line placement, like named-area placement, cannot be resolved on a single
//! node in isolation: a child's `grid-column: sidebar-start / main-end` names lines
//! declared with `[name]` groups on its **container**'s
//! `grid-template-columns` / `grid-template-rows`, so the lines' numbers are only
//! knowable with the tree in hand. The CSS layer therefore carries the child's
//! reference unresolved — a [`GridLineValue::Named`] edge on one of the four
//! `grid-*-start` / `-end` longhands — and
//! [`grid_placement_of`](super::layout_coordinator) maps it to
//! [`GridLine::Auto`]. This pass, which runs after
//! [`layout_styles_from`](super::LayoutCoordinator::layout_styles_from), looks each
//! named edge up against the parent's per-axis line-name map and overwrites just
//! that edge of the item's [`GridPlacement`] with the concrete
//! [`GridLine::Line`] the name resolves to (spec §4.13).
//!
//! A child with no named edge is untouched, and a named edge whose name is absent
//! from the container's template (or whose parent authored no template) is left at
//! its [`GridLine::Auto`] default, so it simply auto-places — keeping every
//! non-named layout byte-for-byte identical. When a line name is declared more than
//! once, the **first** (lowest-numbered) occurrence wins; a trailing `<integer>`
//! disambiguator (`main 2`) is not yet modelled — see the crate follow-ups.

use std::collections::HashMap;

use obscur_layout::algorithm::{GridLine, GridPlacement};
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{GridLineValue, PropertyId, PropertyValue};
use obscur_tree::tree::RetainedTree;

use super::{ComputedStyles, LayoutStyles};

/// Resolves every grid item's named `grid-column` / `grid-row` line references
/// against its container's `grid-template-columns` / `grid-template-rows` line
/// names, patching the item's [`LayoutStyle`] placement in place.
///
/// For each node carrying a [`GridLineValue::Named`] edge, the node's **logical
/// parent** supplies the containing grid; the parent's computed track lists provide
/// the per-axis `name -> line number` map, and each named edge is rewritten to the
/// concrete [`GridLine::Line`] it resolves to. A node without a named edge, without
/// a logical parent, whose parent authored no line names, or that names an unknown
/// line is left unchanged (auto placement).
///
/// This never adds or removes entries: it only overwrites the `grid_placement`
/// field of items already present in `styles`, so it is a pure refinement of the
/// table [`layout_styles_from`](super::LayoutCoordinator::layout_styles_from)
/// built. Patches are independent per node, so the (unordered) iteration is
/// deterministic in its result.
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

/// Whether any of the four `grid-*-start` / `-end` longhands is a
/// [`GridLineValue::Named`] edge (the only case this pass acts on).
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

/// Resolves one named edge against `map`, returning the concrete
/// [`GridLine::Line`] it names; a non-named edge, or a name absent from `map`,
/// keeps `current` (its already-computed value, `GridLine::Auto` for a name).
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

/// Reads one grid-line longhand as its style-local [`GridLineValue`], or [`None`]
/// when the property is absent or not a grid-line value.
fn grid_line_of(style: &ComputedStyle, property: PropertyId) -> Option<&GridLineValue> {
    style
        .get(property)
        .map(ComputedValue::value)
        .and_then(PropertyValue::as_grid_line)
}

/// Builds a `name -> 1-based line number` map from a container's
/// `grid-template-columns` / `grid-template-rows` line names.
///
/// The track list's `line_names[i]` bucket holds the names on grid line `i + 1`
/// (line `0` is the line before the first track). A name declared more than once
/// keeps its **first** (lowest) line number, so a repeated name resolves to its
/// earliest occurrence. An axis with no authored line names yields an empty map.
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
        let mut computed = ComputedStyles::new();
        computed.insert(parent, container_with_named_columns());
        computed.insert(child, item_with_named_columns("b", "c"));

        let mut styles = LayoutStyles::new();
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
        let mut computed = ComputedStyles::new();
        computed.insert(parent, container_with_named_columns());
        computed.insert(child, item_with_named_columns("nope", "c"));

        let mut styles = LayoutStyles::new();
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
        let mut computed = ComputedStyles::new();
        computed.insert(parent, container_with_named_columns());
        computed.insert(child, ComputedStyle::new());

        let mut styles = LayoutStyles::new();
        styles.insert(child, LayoutStyle::default());

        resolve_named_grid_lines(&mut styles, &tree, &computed);

        assert_eq!(
            styles.get(&child).expect("child").grid_placement,
            GridPlacement::default()
        );
    }
}
