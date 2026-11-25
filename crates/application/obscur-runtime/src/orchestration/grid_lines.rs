#![doc = include_str!("../../docs/api/orchestration/grid_lines.doc/_module.md")]

use std::collections::HashMap;

use obscur_layout::algorithm::{GridLine, GridPlacement, RepeatedGridLine};
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{
    GridAreaTemplate, GridLineValue, GridTrack, GridTrackList, PropertyId, PropertyValue,
};
use obscur_tree::tree::RetainedTree;

use super::grid_areas::grid_areas_of;
use super::{ComputedStyles, GridTemplates, LayoutStyles};

#[doc = include_str!("../../docs/api/orchestration/grid_lines.doc/resolve_named_grid_lines.md")]
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
        let Some(parent) = tree.visual_parent_of(node) else {
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
        let areas = grid_areas_of(parent_style);
        let implicit_columns = !has_auto_repeat(parent_style, PropertyId::GridTemplateColumns);
        let implicit_rows = !has_auto_repeat(parent_style, PropertyId::GridTemplateRows);
        let placement = &layout.grid_placement;

        let column_start = resolve_edge(
            style,
            PropertyId::GridColumnStart,
            &columns,
            implicit_columns,
            areas,
            placement.column_start(),
        );
        let column_end = resolve_edge(
            style,
            PropertyId::GridColumnEnd,
            &columns,
            implicit_columns,
            areas,
            placement.column_end(),
        );
        let row_start = resolve_edge(
            style,
            PropertyId::GridRowStart,
            &rows,
            implicit_rows,
            areas,
            placement.row_start(),
        );
        let row_end = resolve_edge(
            style,
            PropertyId::GridRowEnd,
            &rows,
            implicit_rows,
            areas,
            placement.row_end(),
        );

        layout.grid_placement = GridPlacement::new(column_start, column_end, row_start, row_end);
    }
}

/// Saves placement lookups whose line numbers depend on layout's repeat count.
pub(crate) fn defer_repeated_grid_lines(
    templates: &mut GridTemplates,
    tree: &RetainedTree,
    computed: &ComputedStyles,
) {
    if !templates.values().any(|template| {
        template.columns().auto_repeat().is_some() || template.rows().auto_repeat().is_some()
    }) {
        return;
    }
    for (&node, style) in computed {
        if !has_named_edge(style) {
            continue;
        }
        let Some(parent) = tree.visual_parent_of(node) else {
            continue;
        };
        let (Some(template), Some(parent_style)) =
            (templates.get_mut(&parent), computed.get(&parent))
        else {
            continue;
        };
        let areas = grid_areas_of(parent_style);
        let columns = grid_track_list(parent_style, PropertyId::GridTemplateColumns);
        let rows = grid_track_list(parent_style, PropertyId::GridTemplateRows);
        let line = |property, list: Option<&GridTrackList>| {
            repeated_edge(grid_line_of(style, property)?, list?, areas, property)
        };
        let lines = [
            line(PropertyId::GridColumnStart, columns),
            line(PropertyId::GridColumnEnd, columns),
            line(PropertyId::GridRowStart, rows),
            line(PropertyId::GridRowEnd, rows),
        ];
        if lines.iter().any(Option::is_some) {
            template.set_repeated_lines(node, lines);
        }
    }
}

fn repeated_edge(
    value: &GridLineValue,
    list: &GridTrackList,
    areas: Option<&GridAreaTemplate>,
    property: PropertyId,
) -> Option<RepeatedGridLine> {
    let (name, occurrence) = match value {
        GridLineValue::Named(name) => (name, std::num::NonZeroI32::new(1)?),
        GridLineValue::NamedOccurrence(name, occurrence) => (name, *occurrence),
        _ => return None,
    };
    let (start, pattern) =
        list.tracks()
            .iter()
            .enumerate()
            .find_map(|(index, track)| match track {
                GridTrack::AutoRepeat { line_names, .. } => Some((index, line_names)),
                _ => None,
            })?;
    let lookup = |name: &str, count_implicit| {
        let matching_lines = |buckets: &[Box<[Box<str>]>]| {
            buckets
                .iter()
                .enumerate()
                .filter_map(|(index, names)| {
                    names
                        .iter()
                        .any(|candidate| candidate.as_ref() == name)
                        .then_some(index)
                })
                .collect::<Vec<_>>()
        };
        let mut fixed = Vec::new();
        let mut after = Vec::new();
        for index in matching_lines(list.line_names()) {
            if index <= start {
                fixed.push(index);
            } else {
                after.push(index - start - 1);
            }
        }
        if let Some(line) = areas.and_then(|template| area_edge(template, property, name)) {
            fixed.push((line - 1) as usize);
        }
        RepeatedGridLine::new(
            occurrence,
            fixed,
            matching_lines(pattern),
            after,
            count_implicit,
        )
    };
    let direct = lookup(name, true);
    if matches!(value, GridLineValue::Named(_)) {
        let preferred = lookup(&area_edge_name(property, name), false);
        return Some(direct.with_preferred(preferred));
    }
    Some(direct)
}

fn grid_track_list(style: &ComputedStyle, property: PropertyId) -> Option<&GridTrackList> {
    style
        .get(property)
        .map(ComputedValue::value)
        .and_then(PropertyValue::as_grid_track_list)
}

fn has_auto_repeat(style: &ComputedStyle, property: PropertyId) -> bool {
    grid_track_list(style, property).is_some_and(|list| {
        list.tracks()
            .iter()
            .any(|track| matches!(track, GridTrack::AutoRepeat { .. }))
    })
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
    .any(|id| {
        matches!(
            grid_line_of(style, id),
            Some(GridLineValue::Named(_) | GridLineValue::NamedOccurrence(..))
        )
    })
}

/// Resolves a named edge against authored and area-generated line names.
/// Bare names prefer the corresponding area-edge spelling. Numbered names
/// count distinct matching lines from the requested end. When overtaking
/// the matches in the explicit grid, counting continues on implicit lines.
fn resolve_edge(
    style: &ComputedStyle,
    property: PropertyId,
    map: &HashMap<&str, Vec<i32>>,
    allow_implicit: bool,
    areas: Option<&GridAreaTemplate>,
    current: GridLine,
) -> GridLine {
    if !allow_implicit {
        return current; // Layout resolves this axis after expanding auto-repeat.
    }
    let (name, occurrence) = match grid_line_of(style, property) {
        Some(GridLineValue::Named(name)) => {
            let preferred = area_edge_name(property, name);
            let first = map
                .get(preferred.as_str())
                .and_then(|matches| matches.first())
                .copied();
            let generated = areas.and_then(|template| area_edge(template, property, &preferred));
            if let Some(line) = first.into_iter().chain(generated).min() {
                return GridLine::Line(line);
            }
            (name, 1)
        }
        Some(GridLineValue::NamedOccurrence(name, occurrence)) => (name, occurrence.get()),
        _ => return current,
    };
    let explicit = map.get(name.as_ref()).map_or(&[][..], Vec::as_slice);
    let implicit = areas.and_then(|template| area_edge(template, property, name));
    let extra = implicit.filter(|line| explicit.binary_search(line).is_err());
    let count = explicit.len() + usize::from(extra.is_some());
    let Ok(occurrence_count) = usize::try_from(occurrence.unsigned_abs()) else {
        return current;
    };
    let offset = occurrence_count - 1;
    if offset >= count {
        return u32::try_from(occurrence_count - count).map_or(current, |missing| {
            GridLine::outside_explicit(missing, occurrence < 0)
        });
    }
    let index = if occurrence < 0 {
        count - 1 - offset
    } else {
        offset
    };
    if let Some(line) = extra {
        let insertion = explicit.partition_point(|&explicit_line| explicit_line < line);
        if index == insertion {
            return GridLine::Line(line);
        }
        return GridLine::Line(explicit[index - usize::from(index > insertion)]);
    }
    GridLine::Line(explicit[index])
}

/// Converts a bare name to the spelling used by the requested area edge.
fn area_edge_name(property: PropertyId, name: &str) -> String {
    let suffix = match property {
        PropertyId::GridColumnEnd | PropertyId::GridRowEnd => "end",
        _ => "start",
    };
    format!("{name}-{suffix}")
}

/// Finds the line contributed by a template area's generated name.
fn area_edge(template: &GridAreaTemplate, property: PropertyId, name: &str) -> Option<i32> {
    let (area, is_start) = name
        .strip_suffix("-start")
        .map(|area| (area, true))
        .or_else(|| name.strip_suffix("-end").map(|area| (area, false)))?;
    let area = template.area(area)?;
    let line = match (property, is_start) {
        (PropertyId::GridColumnStart | PropertyId::GridColumnEnd, true) => area.column_start,
        (PropertyId::GridColumnStart | PropertyId::GridColumnEnd, false) => area.column_end,
        (_, true) => area.row_start,
        (_, false) => area.row_end,
    };
    Some(i32::try_from(line).unwrap_or(i32::MAX))
}

/// Reads one grid-line longhand as its style-local [`GridLineValue`], or [`None`]
/// when the property is absent or not a grid-line value.
fn grid_line_of(style: &ComputedStyle, property: PropertyId) -> Option<&GridLineValue> {
    style
        .get(property)
        .map(ComputedValue::value)
        .and_then(PropertyValue::as_grid_line)
}

#[doc = include_str!("../../docs/api/orchestration/grid_lines.doc/line_name_map.md")]
fn line_name_map(style: &ComputedStyle, property: PropertyId) -> HashMap<&str, Vec<i32>> {
    let mut map = HashMap::<&str, Vec<i32>>::new();
    let Some(list) = grid_track_list(style, property) else {
        return map;
    };
    if list
        .tracks()
        .iter()
        .any(|track| matches!(track, GridTrack::AutoRepeat { .. }))
    {
        return map;
    }
    for (index, names) in list.line_names().iter().enumerate() {
        let line = i32::try_from(index).unwrap_or(i32::MAX).saturating_add(1);
        for name in names {
            let lines = map.entry(name.as_ref()).or_default();
            if lines.last() != Some(&line) {
                lines.push(line);
            }
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
    fn unknown_line_name_resolves_to_an_implicit_track() {
        let (tree, parent, child) = tree_with_child();
        let mut computed = ComputedStyles::default();
        computed.insert(parent, container_with_named_columns());
        computed.insert(child, item_with_named_columns("nope", "c"));

        let mut styles = LayoutStyles::default();
        styles.insert(child, LayoutStyle::default());

        resolve_named_grid_lines(&mut styles, &tree, &computed);

        let placement = styles.get(&child).expect("child").grid_placement;
        assert_eq!(
            placement.column_start(),
            GridLine::outside_explicit(1, false)
        );
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
