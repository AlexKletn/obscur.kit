//! `Table`: a headless columns/rows data table with single-column sort
//! (spec §2.6).

use super::table_row::TableRow;
use super::{Column, SortDirection};

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Text;
use obscur_view::view::{LOCAL_NAME, STYLE, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("../../docs/api/table/table.doc/Table.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    columns: Vec<Column>,
    column_order: Vec<usize>,
    rows: Vec<TableRow>,
    sort: Option<(usize, SortDirection)>,
    sticky_header: bool,
}

impl Table {
    #[doc = include_str!("../../docs/api/table/table.doc/COMPONENT_ID.md")]
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_000A);

    #[doc = include_str!("../../docs/api/table/table.doc/TABLE_ROW_ID.md")]
    const TABLE_ROW_ID: ComponentId = ComponentId::from_raw(0x00D0_001A);
    #[doc = include_str!("../../docs/api/table/table.doc/TABLE_CELL_ID.md")]
    const TABLE_CELL_ID: ComponentId = ComponentId::from_raw(0x00D0_002A);

    #[doc = include_str!("../../docs/api/table/table.doc/PROP_SORT_COL.md")]
    const PROP_SORT_COL: InternedSymbol = InternedSymbol::from_raw(0x00D1_0A01);
    /// Interned name of the per-header-cell `sortable` state property.
    const PROP_SORTABLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0A02);

    /// Creates an empty, unsorted table.
    #[must_use]
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            column_order: Vec::new(),
            rows: Vec::new(),
            sort: None,
            sticky_header: false,
        }
    }

    /// Appends a single column, returning the updated table (fluent builder).
    #[must_use]
    pub fn column(mut self, column: Column) -> Self {
        self.column_order.push(self.columns.len());
        self.columns.push(column);
        self
    }

    /// Appends every column from `columns`, returning the updated table.
    #[must_use]
    pub fn with_columns<I>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = Column>,
    {
        for column in columns {
            self = self.column(column);
        }
        self
    }

    /// Appends a single data row (one cell string per column), returning the
    /// updated table (fluent builder).
    #[must_use]
    pub fn row<I, S>(mut self, cells: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Box<str>>,
    {
        self.rows.push(TableRow {
            key: self.rows.len() + 1,
            cells: cells.into_iter().map(Into::into).collect(),
        });
        self
    }

    /// Appends every row from `rows`, returning the updated table.
    #[must_use]
    pub fn with_rows<I, R, S>(mut self, rows: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: IntoIterator<Item = S>,
        S: Into<Box<str>>,
    {
        for row in rows {
            self = self.row(row);
        }
        self
    }

    /// Moves a data row to the requested final index while preserving its identity. Both indices exclude the header. A successful move clears the sort indicator.
    pub fn move_row(&mut self, from: usize, to: usize) -> bool {
        if from == to || from >= self.rows.len() || to >= self.rows.len() {
            return false;
        }
        let row = self.rows.remove(from);
        self.rows.insert(to, row);
        self.sort = None;
        true
    }

    /// Applies an inline CSS sticky header with a top inset of zero. It follows the runtime viewport’s sticky-positioning path.
    #[must_use]
    pub fn sticky_header(mut self, sticky: bool) -> Self {
        self.sticky_header = sticky;
        self
    }

    /// Moves a column to the requested final visual index. Header, cells, width, and sort identity move together.
    pub fn move_column(&mut self, from: usize, to: usize) -> bool {
        if from == to || from >= self.column_order.len() || to >= self.column_order.len() {
            return false;
        }
        let column = self.column_order.remove(from);
        self.column_order.insert(to, column);
        true
    }

    /// Sets a visual column width in logical pixels. None restores automatic sizing.
    pub fn resize_column(&mut self, column: usize, width: Option<u32>) -> bool {
        let Some(&key) = self.column_order.get(column) else {
            return false;
        };
        self.columns[key].width = width;
        true
    }

    /// Returns the explicit width of the column at the visual index.
    #[must_use]
    pub fn column_width(&self, column: usize) -> Option<u32> {
        self.columns[*self.column_order.get(column)?].width
    }

    /// Sorts the rows by the cell text of column `column`, returning `true` on
    /// success. Sorting by a fresh column sorts ascending; sorting by the
    /// already-active column flips the direction. The sort is **stable**.
    /// Returns `false` (leaving state untouched) when `column` is out of range or
    /// its column is not [`sortable`](Column::is_sortable).
    pub fn sort_by(&mut self, column: usize) -> bool {
        let Some(&column) = self.column_order.get(column) else {
            return false;
        };
        if !self.columns[column].is_sortable() {
            return false;
        }
        let direction = match self.sort {
            Some((active, dir)) if active == column => dir.reversed(),
            _ => SortDirection::Ascending,
        };
        self.sort = Some((column, direction));
        self.rows.sort_by(|a, b| {
            let left = a.cells.get(column).map_or("", |c| &**c);
            let right = b.cells.get(column).map_or("", |c| &**c);
            let ordering = left.cmp(right);
            match direction {
                SortDirection::Ascending => ordering,
                SortDirection::Descending => ordering.reverse(),
            }
        });
        true
    }

    /// Returns the index of the sorted column, if any.
    #[must_use]
    pub fn sorted_column(&self) -> Option<usize> {
        let (key, _) = self.sort?;
        self.column_order.iter().position(|column| *column == key)
    }

    /// Returns the active sort direction, if any.
    #[must_use]
    pub fn sort_direction(&self) -> Option<SortDirection> {
        self.sort.map(|(_, direction)| direction)
    }

    /// Returns the number of data rows.
    #[must_use]
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Returns the number of columns.
    #[must_use]
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Returns whether the table has no columns and no rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty() && self.rows.is_empty()
    }

    /// Returns the accessible role of this control ([`SemanticRole::Table`]).
    #[must_use]
    pub const fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Table
    }

    #[doc = include_str!("../../docs/api/table/table.doc/view.md")]
    #[must_use]
    pub fn view(&self) -> View {
        self.view_rows("table", 0..self.rows.len(), |_, row| row)
    }

    /// Renders a table-like view from a subset of logical rows while preserving their insertion keys.
    pub(crate) fn view_rows(
        &self,
        local_name: &'static str,
        row_indices: impl IntoIterator<Item = usize>,
        mut configure_row: impl FnMut(usize, ViewNode) -> ViewNode,
    ) -> View {
        let rows = std::iter::once(self.header_row(None)).chain(
            row_indices
                .into_iter()
                .map(|index| configure_row(index, self.data_row(index))),
        );
        self.view_with_children(local_name, rows.collect())
    }

    /// Builds the keyed header with the current column order.
    pub(crate) fn header_row(&self, height: Option<u32>) -> ViewNode {
        // The table's parts name themselves in HTML's own vocabulary, under the
        // reserved LOCAL_NAME symbol the matcher reads as an element's *type*
        // (spec §4.13). They used to carry a private `role` property instead — not
        // a class, not a local name, invisible to CSS — so the UA sheet's `table`,
        // `th`, `td` and `tr:hover` rules were dead against this widget while
        // matching the same markup written by hand. A header cell is `th` and a
        // data cell `td`, the distinction the shared `TABLE_CELL_ID` cannot carry
        // and the one the sheet styles differently.
        let header_cells: Vec<ViewNode> = self
            .column_order
            .iter()
            .map(|&key| {
                let column = &self.columns[key];
                self.cell(key, "th", &column.title)
                    .with_prop(ViewProperty::new(
                        Self::PROP_SORTABLE,
                        ViewValue::Bool(column.sortable),
                    ))
            })
            .collect();
        let mut header = ViewNode::element(Self::TABLE_ROW_ID)
            .with_key(0_usize)
            .with_prop(ViewProperty::new(LOCAL_NAME, ViewValue::from("tr")))
            .with_children(ViewFragment::from_nodes(header_cells));

        let style = if let Some(height) = height {
            Some(format!("box-sizing:border-box;height:{height}px"))
        } else if self.sticky_header {
            Some("position:sticky;top:0;z-index:1".into())
        } else {
            None
        };
        if let Some(style) = style {
            header = header.with_prop(ViewProperty::new(STYLE, ViewValue::from(style)));
        }

        header
    }

    /// Builds a single keyed data row at its logical index.
    pub(crate) fn data_row(&self, index: usize) -> ViewNode {
        let row = &self.rows[index];
        let cells: Vec<ViewNode> = self
            .column_order
            .iter()
            .copied()
            .chain(self.columns.len()..row.cells.len())
            .map(|key| self.cell(key, "td", row.cells.get(key).map_or("", Box::as_ref)))
            .collect();
        ViewNode::element(Self::TABLE_ROW_ID)
            .with_key(row.key)
            .with_prop(ViewProperty::new(LOCAL_NAME, ViewValue::from("tr")))
            .with_children(ViewFragment::from_nodes(cells))
    }

    /// Builds the table container with rows or virtual list markers as its children.
    pub(crate) fn view_with_children(&self, local_name: &'static str, rows: Vec<ViewNode>) -> View {
        let sort_col = self
            .sorted_column()
            .map_or(-1, |i| i64::try_from(i).unwrap_or(-1));

        let node = ViewNode::element(Self::COMPONENT_ID)
            .with_prop(ViewProperty::new(LOCAL_NAME, ViewValue::from(local_name)))
            .with_prop(ViewProperty::new(
                Self::PROP_SORT_COL,
                ViewValue::Int(sort_col),
            ))
            .with_children(ViewFragment::from_nodes(rows));
        View::single(node)
    }

    /// Creates a keyed cell with any explicit column sizing.
    fn cell(&self, key: usize, tag: &'static str, text: &str) -> ViewNode {
        let mut node = ViewNode::element(Self::TABLE_CELL_ID)
            .with_key(key)
            .with_prop(ViewProperty::new(LOCAL_NAME, ViewValue::from(tag)))
            .with_children(ViewFragment::from_nodes(vec![
                Text::new(text).into_view_node(),
            ]));
        if let Some(width) = self.columns.get(key).and_then(|column| column.width) {
            let style = format!(
                "box-sizing:border-box;width:{width}px;min-width:{width}px;max-width:{width}px;flex-shrink:0"
            );
            node = node.with_prop(ViewProperty::new(STYLE, ViewValue::from(style)));
        }
        node
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Table {
        Table::new()
            .with_columns([Column::new("Name").sortable(true), Column::new("Note")])
            .row(["Charlie", "c"])
            .row(["Alice", "a"])
            .row(["Bob", "b"])
    }

    #[test]
    fn column_builder() {
        let c = Column::new("X").sortable(true);
        assert_eq!(c.title(), "X");
        assert!(c.is_sortable());
    }

    #[test]
    fn counts() {
        let t = sample();
        assert_eq!(t.column_count(), 2);
        assert_eq!(t.row_count(), 3);
        assert!(!t.is_empty());
    }

    #[test]
    fn sort_orders_rows_ascending() {
        let mut t = sample();
        assert!(t.sort_by(0));
        assert_eq!(t.sorted_column(), Some(0));
        assert_eq!(t.sort_direction(), Some(SortDirection::Ascending));
        assert_eq!(&*t.rows[0].cells[0], "Alice");
        assert_eq!(&*t.rows[2].cells[0], "Charlie");
    }

    #[test]
    fn sort_again_flips_direction() {
        let mut t = sample();
        assert!(t.sort_by(0));
        assert!(t.sort_by(0));
        assert_eq!(t.sort_direction(), Some(SortDirection::Descending));
        assert_eq!(&*t.rows[0].cells[0], "Charlie");
        assert_eq!(&*t.rows[2].cells[0], "Alice");
    }

    #[test]
    fn sort_non_sortable_or_out_of_range_fails() {
        let mut t = sample();
        assert!(!t.sort_by(1)); // not sortable
        assert!(!t.sort_by(9)); // out of range
        assert_eq!(t.sorted_column(), None);
    }

    #[test]
    fn semantic_role_is_table() {
        assert_eq!(sample().semantic_role(), SemanticRole::Table);
    }

    fn local_name_of(node: &ViewNode) -> Option<String> {
        let ViewNode::Element { props, .. } = node else {
            return None;
        };
        props
            .iter()
            .find(|p| p.name() == LOCAL_NAME)
            .and_then(|p| match p.value() {
                ViewValue::Text(text) => Some(text.to_string()),
                _ => None,
            })
    }

    #[test]
    fn every_part_names_itself_in_the_html_table_vocabulary() {
        // What the UA sheet's `table`, `th`, `td` and `tr:hover` rules match. The
        // parts used to carry a private `role` property — invisible to CSS — so
        // those four rules were dead against this widget, and a hand-written
        // `<table>` and a `Table::new()` were two different trees under one sheet.
        let view = sample().view();
        let root = &view.root().nodes()[0];
        assert_eq!(local_name_of(root).as_deref(), Some("table"));

        let ViewNode::Element { children: rows, .. } = root else {
            panic!("expected element root");
        };
        for row in rows.nodes() {
            assert_eq!(local_name_of(row).as_deref(), Some("tr"));
        }

        // The header row's cells are `th`, every data row's are `td` — the one
        // distinction the shared `TABLE_CELL_ID` cannot carry, and the one the
        // sheet styles differently.
        let cells_of = |row: &ViewNode| {
            let ViewNode::Element { children, .. } = row else {
                panic!("expected element row");
            };
            children
                .nodes()
                .iter()
                .map(local_name_of)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            cells_of(&rows.nodes()[0]),
            vec![Some("th".to_owned()), Some("th".to_owned())]
        );
        for data_row in &rows.nodes()[1..] {
            assert_eq!(
                cells_of(data_row),
                vec![Some("td".to_owned()), Some("td".to_owned())]
            );
        }
    }

    #[test]
    fn view_has_header_and_data_rows() {
        let t = sample();
        let view = t.view();
        match &view.root().nodes()[0] {
            ViewNode::Element {
                component,
                props,
                children,
                ..
            } => {
                assert_eq!(*component, Table::COMPONENT_ID);
                // header row + 3 data rows
                assert_eq!(children.len(), 4);
                let sort_col = props
                    .iter()
                    .find(|p| p.name() == Table::PROP_SORT_COL)
                    .expect("sort-col prop present");
                assert_eq!(*sort_col.value(), ViewValue::Int(-1));
                // Header row has a cell per column, first carries the sortable flag.
                match &children.nodes()[0] {
                    ViewNode::Element {
                        children: cells, ..
                    } => {
                        assert_eq!(cells.len(), 2);
                        match &cells.nodes()[0] {
                            ViewNode::Element { props, .. } => {
                                let sortable = props
                                    .iter()
                                    .find(|p| p.name() == Table::PROP_SORTABLE)
                                    .expect("sortable prop present");
                                assert_eq!(*sortable.value(), ViewValue::Bool(true));
                            }
                            ViewNode::Text { .. } => panic!("expected element cell"),
                        }
                    }
                    ViewNode::Text { .. } => panic!("expected element row"),
                }
            }
            ViewNode::Text { .. } => panic!("expected element root"),
        }
    }
}
