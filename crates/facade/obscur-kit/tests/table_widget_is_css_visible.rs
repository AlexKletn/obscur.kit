#![doc = include_str!("table_widget_is_css_visible.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::default_stylesheet::ua_stylesheet;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::computed::ComputedValue;
use obscur_kit::style::property::PropertyId;
use obscur_kit::style::selector::PseudoState;
use obscur_kit::ui_macros::okui;
use obscur_kit::view::view::View;
use obscur_kit::widgets::table::{Column, Table};

#[doc = include_str!("table_widget_is_css_visible.doc/PAPER.md")]
const PAPER: (f32, f32, f32) = (
    0xfa as f32 / 255.0,
    0xf9 as f32 / 255.0,
    0xf6 as f32 / 255.0,
);
#[doc = include_str!("table_widget_is_css_visible.doc/INK.md")]
const INK: (f32, f32, f32) = (
    0x2b as f32 / 255.0,
    0x2a as f32 / 255.0,
    0x27 as f32 / 255.0,
);
#[doc = include_str!("table_widget_is_css_visible.doc/HOVER.md")]
const HOVER: (f32, f32, f32) = (
    0xe4 as f32 / 255.0,
    0xe2 as f32 / 255.0,
    0xdb as f32 / 255.0,
);

#[doc = include_str!("table_widget_is_css_visible.doc/sample.md")]
fn sample() -> Table {
    Table::new()
        .with_columns([Column::new("Name").sortable(true), Column::new("Note")])
        .row(["Alice", "a"])
        .row(["Bob", "b"])
}

#[doc = include_str!("table_widget_is_css_visible.doc/rendered.md")]
fn rendered(view: &View) -> UiRuntime {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(600.0, 400.0));
    runtime.set_default_stylesheet(ua_stylesheet());
    runtime.render_now(view, &[]);
    runtime
}

#[doc = include_str!("table_widget_is_css_visible.doc/at.md")]
fn at(runtime: &UiRuntime, path: &[usize]) -> NodeId {
    let mut node = runtime.tree().root().expect("a root");
    for &index in path {
        node = runtime.tree().get(node).expect("a live node").child_nodes()[index];
    }
    node
}

#[doc = include_str!("table_widget_is_css_visible.doc/local_name.md")]
fn local_name(runtime: &UiRuntime, node: NodeId) -> Option<String> {
    runtime
        .tree()
        .get(node)
        .and_then(|record| record.attributes().local_name())
        .map(str::to_owned)
}

#[doc = include_str!("table_widget_is_css_visible.doc/colour.md")]
fn colour(runtime: &UiRuntime, node: NodeId, property: PropertyId) -> Option<(f32, f32, f32)> {
    runtime
        .computed_style(node)
        .and_then(|style| style.get(property))
        .and_then(ComputedValue::as_color)
        .map(|c| (c.r, c.g, c.b))
}

#[test]
fn the_widgets_parts_name_themselves_in_htmls_table_vocabulary() {
    // The names are the whole fix. The rows and cells are real element nodes and
    // always were, but they carried a PRIVATE `role` property whose text was
    // "row"/"cell" — not a class, not a local name, and so invisible to matching.
    let runtime = rendered(&sample().view());
    assert_eq!(
        local_name(&runtime, at(&runtime, &[0])).as_deref(),
        Some("table")
    );
    // Header row, then one row per data row — every one of them a `tr`.
    for row in 0..3 {
        assert_eq!(
            local_name(&runtime, at(&runtime, &[0, row])).as_deref(),
            Some("tr"),
            "row {row}"
        );
    }
    // The header's cells are `th` and the data rows' are `td`: the one distinction
    // the shared cell component id cannot carry, and the one the sheet styles
    // differently.
    for column in 0..2 {
        assert_eq!(
            local_name(&runtime, at(&runtime, &[0, 0, column])).as_deref(),
            Some("th"),
            "header cell {column}"
        );
        for row in 1..3 {
            assert_eq!(
                local_name(&runtime, at(&runtime, &[0, row, column])).as_deref(),
                Some("td"),
                "cell {row}/{column}"
            );
        }
    }
}

#[test]
fn the_ua_sheets_th_and_td_rules_reach_the_widgets_own_cells() {
    // THE claim, and it was false: `th { background: #2b2a27; color: #faf9f6 }` and
    // `td { color: #2b2a27 }` are a third of the UA sheet, and against the widget
    // path they matched nothing at all.
    let runtime = rendered(&sample().view());
    let header_cell = at(&runtime, &[0, 0, 0]);
    let data_cell = at(&runtime, &[0, 1, 0]);

    assert_eq!(
        colour(&runtime, header_cell, PropertyId::BackgroundColor),
        Some(INK),
        "the `th` rule's reversed header fill"
    );
    assert_eq!(
        colour(&runtime, header_cell, PropertyId::Color),
        Some(PAPER),
        "…and its reversed ink"
    );
    assert_eq!(
        colour(&runtime, data_cell, PropertyId::Color),
        Some(INK),
        "the `td` rule's ordinary ink"
    );
    assert_ne!(
        colour(&runtime, data_cell, PropertyId::BackgroundColor),
        colour(&runtime, header_cell, PropertyId::BackgroundColor),
        "a data cell takes no header fill — `th` and `td` are two rules"
    );
}

#[test]
fn the_ua_sheets_tr_hover_rule_reaches_a_row() {
    // The fourth table rule, and the one that needs a state to show: a row is a
    // `tr`, so hovering one applies `tr:hover { background: #e4e2db }`. It could
    // not match a node with no local name however the pointer moved.
    let mut runtime = rendered(&sample().view());
    let row = at(&runtime, &[0, 1]);
    assert_ne!(
        colour(&runtime, row, PropertyId::BackgroundColor),
        Some(HOVER)
    );

    runtime.set_pseudo_state(row, PseudoState::HOVER);
    runtime.render_now(&sample().view(), &[]);
    assert_eq!(
        colour(&runtime, row, PropertyId::BackgroundColor),
        Some(HOVER),
        "`tr:hover` must reach a row the widget built"
    );
    // And only that row: the sibling below it is untouched, so this is the row's
    // own state and not a rule that reaches the whole table.
    assert_ne!(
        colour(&runtime, at(&runtime, &[0, 2]), PropertyId::BackgroundColor),
        Some(HOVER)
    );
}

#[test]
fn the_widget_and_hand_written_markup_are_styled_by_the_same_rules() {
    // "One sheet, one vocabulary, one result." The `<table>` an author writes and
    // the `Table` a program builds were two different trees under the same four
    // rules — the markup matched them and the widget did not. Same computed values
    // now, from the same sheet, with nothing widget-specific in it.
    let widget = rendered(&sample().view());
    let markup = rendered(&okui! {
        <table>
            <tr><th>"Name"</th></tr>
            <tr><td>"Alice"</td></tr>
        </table>
    });

    for property in [PropertyId::BackgroundColor, PropertyId::Color] {
        assert_eq!(
            colour(&widget, at(&widget, &[0, 0, 0]), property),
            colour(&markup, at(&markup, &[0, 0, 0]), property),
            "header cell {property:?}"
        );
        assert_eq!(
            colour(&widget, at(&widget, &[0, 1, 0]), property),
            colour(&markup, at(&markup, &[0, 1, 0]), property),
            "data cell {property:?}"
        );
    }
}

#[test]
fn sorting_and_moving_table_rows_preserve_their_nodes() {
    let mut table = sample();
    let mut rt = rendered(&table.view());
    let header = at(&rt, &[0, 0]);
    let alice = at(&rt, &[0, 1]);
    let bob = at(&rt, &[0, 2]);

    assert!(table.sort_by(0));
    assert!(table.sort_by(0));
    let _ = rt.render_now(&table.view(), &[]);
    assert_eq!(at(&rt, &[0, 0]), header);
    assert_eq!(at(&rt, &[0, 1]), bob);
    assert_eq!(at(&rt, &[0, 2]), alice);

    assert!(!table.move_row(2, 0));
    assert!(table.move_row(0, 1));
    assert!(table.sorted_column().is_none());
    let _ = rt.render_now(&table.view(), &[]);
    assert_eq!(at(&rt, &[0, 0]), header);
    assert_eq!(at(&rt, &[0, 1]), alice);
    assert_eq!(at(&rt, &[0, 2]), bob);
}

#[test]
fn moving_and_resizing_a_column_keeps_its_cells_and_sort_state() {
    use obscur_kit::bootstrap::CssPipeline;

    let mut table = sample();
    assert!(table.sort_by(0));
    let mut rt = rendered(&table.view());
    CssPipeline::new().install(&mut rt, &[]);
    let header = at(&rt, &[0, 0, 0]);
    let first_cell = at(&rt, &[0, 1, 0]);
    assert!(table.resize_column(0, Some(85)));
    assert!(table.move_column(0, 1));
    assert_eq!(table.sorted_column(), Some(1));
    assert_eq!(table.column_width(1), Some(85));

    let _ = rt.render_now(&table.view(), &[]);
    assert_eq!(at(&rt, &[0, 0, 1]), header);
    assert_eq!(at(&rt, &[0, 1, 1]), first_cell);
    assert_eq!(rt.layout_boxes()[&header].size().width, 85.0);
    assert_eq!(rt.layout_boxes()[&first_cell].size().width, 85.0);

    assert!(table.resize_column(1, None));
    assert_eq!(table.column_width(1), None);
    assert!(!table.move_column(1, 5));
}

#[test]
fn tree_grid_keeps_visible_rows_keyed_and_publishes_hierarchy_state() {
    use obscur_kit::accessibility::semantic::SemanticRole;
    use obscur_kit::interaction::command::NamedKey;
    use obscur_kit::widgets::tree::{TreeGrid, TreeNode};

    let mut grid = TreeGrid::new()
        .column(Column::new("Name"))
        .column(Column::new("Value"))
        .row(TreeNode::new("Parent", 0, true), ["folder"])
        .row(TreeNode::new("Child", 1, false), ["file"])
        .row(TreeNode::new("Sibling", 0, false), ["file"]);
    let mut rt = rendered(&grid.view());
    let root = at(&rt, &[0]);
    let parent = at(&rt, &[0, 1]);
    let sibling = at(&rt, &[0, 2]);
    let snapshot = rt.semantic_snapshot();
    assert_eq!(snapshot.get(root).unwrap().role(), SemanticRole::TreeGrid);
    assert_eq!(snapshot.get(parent).unwrap().state().expanded, Some(false));

    assert!(!grid.select(1));
    assert!(grid.move_selection(1));
    assert_eq!(grid.selected_index(), Some(0));
    assert!(grid.navigate(NamedKey::ArrowRight));
    assert!(grid.navigate(NamedKey::ArrowRight));
    assert_eq!(grid.selected_index(), Some(1));
    let _ = rt.render_now(&grid.view(), &[]);
    let child = at(&rt, &[0, 2]);
    assert_eq!(at(&rt, &[0, 3]), sibling);
    let snapshot = rt.semantic_snapshot();
    assert_eq!(snapshot.get(parent).unwrap().state().expanded, Some(true));
    assert!(snapshot.get(child).unwrap().state().selected);

    assert!(grid.navigate(NamedKey::ArrowLeft));
    assert_eq!(grid.selected_index(), Some(0));
    assert!(grid.navigate(NamedKey::ArrowLeft));
    assert_eq!(grid.selected_index(), Some(0));
    assert!(grid.navigate(NamedKey::End));
    assert_eq!(grid.selected_index(), Some(2));
    let _ = rt.render_now(&grid.view(), &[]);
    assert!(!rt.tree().contains(child));
    assert_eq!(at(&rt, &[0, 2]), sibling);
}

#[test]
fn a_sticky_table_header_uses_the_viewport_scroll_geometry() {
    use obscur_kit::bootstrap::CssPipeline;
    use obscur_kit::style::property::layout_keyword::POSITION_STICKY;

    let table = Table::new()
        .column(Column::new("Name"))
        .with_rows((0..30).map(|_| ["row"]))
        .sticky_header(true);
    let view = table.view();
    let mut rt = rendered(&view);
    CssPipeline::new().install(&mut rt, &[]);
    let _ = rt.render_now(&view, &[]);
    let header = at(&rt, &[0, 0]);
    let before = rt.layout_boxes()[&header].origin().y;
    assert_eq!(
        rt.computed_style(header)
            .unwrap()
            .get(PropertyId::Position)
            .unwrap()
            .value(),
        &obscur_kit::style::property::PropertyValue::Keyword(POSITION_STICKY),
    );
    rt.set_viewport_scroll(0.0, 30.0);
    let _ = rt.render_now(&view, &[]);
    let bounds = rt.layout_boxes()[&header].bounds;
    assert!(bounds.origin.y > before);
    let point = obscur_kit::geometry::Point::new(bounds.origin.x + 5.0, bounds.origin.y + 5.0);
    assert!(rt.hit_test(point).contains_node(header));
}
