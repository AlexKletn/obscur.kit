#![doc = include_str!("table.doc/_module.md")]

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Text;
use obscur_view::view::{LOCAL_NAME, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("table.doc/Column.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    title: Box<str>,
    sortable: bool,
}

impl Column {
    #[doc = include_str!("table.doc/new.md")]
    #[must_use]
    pub fn new(title: impl Into<Box<str>>) -> Self {
        Self {
            title: title.into(),
            sortable: false,
        }
    }

    #[doc = include_str!("table.doc/sortable.md")]
    #[must_use]
    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    #[doc = include_str!("table.doc/title.md")]
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[doc = include_str!("table.doc/is_sortable.md")]
    #[must_use]
    pub const fn is_sortable(&self) -> bool {
        self.sortable
    }
}

#[doc = include_str!("table.doc/SortDirection.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    #[doc = include_str!("table.doc/Ascending.md")]
    Ascending,
    #[doc = include_str!("table.doc/Descending.md")]
    Descending,
}

impl SortDirection {
    #[doc = include_str!("table.doc/reversed.md")]
    #[must_use]
    pub const fn reversed(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

#[doc = include_str!("table.doc/Table.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    columns: Vec<Column>,
    rows: Vec<Vec<Box<str>>>,
    sort: Option<(usize, SortDirection)>,
}

impl Table {
    #[doc = include_str!("table.doc/COMPONENT_ID.md")]
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_000A);

    #[doc = include_str!("table.doc/TABLE_ROW_ID.md")]
    const TABLE_ROW_ID: ComponentId = ComponentId::from_raw(0x00D0_001A);
    #[doc = include_str!("table.doc/TABLE_CELL_ID.md")]
    const TABLE_CELL_ID: ComponentId = ComponentId::from_raw(0x00D0_002A);

    #[doc = include_str!("table.doc/PROP_SORT_COL.md")]
    const PROP_SORT_COL: InternedSymbol = InternedSymbol::from_raw(0x00D1_0A01);
    #[doc = include_str!("table.doc/PROP_SORTABLE.md")]
    const PROP_SORTABLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0A02);

    #[doc = include_str!("table.doc/new_2.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            sort: None,
        }
    }

    #[doc = include_str!("table.doc/column.md")]
    #[must_use]
    pub fn column(mut self, column: Column) -> Self {
        self.columns.push(column);
        self
    }

    #[doc = include_str!("table.doc/with_columns.md")]
    #[must_use]
    pub fn with_columns<I>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = Column>,
    {
        self.columns.extend(columns);
        self
    }

    #[doc = include_str!("table.doc/row.md")]
    #[must_use]
    pub fn row<I, S>(mut self, cells: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Box<str>>,
    {
        self.rows.push(cells.into_iter().map(Into::into).collect());
        self
    }

    #[doc = include_str!("table.doc/with_rows.md")]
    #[must_use]
    pub fn with_rows<I, R, S>(mut self, rows: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: IntoIterator<Item = S>,
        S: Into<Box<str>>,
    {
        for row in rows {
            self.rows.push(row.into_iter().map(Into::into).collect());
        }
        self
    }

    #[doc = include_str!("table.doc/sort_by.md")]
    pub fn sort_by(&mut self, column: usize) -> bool {
        if !self.columns.get(column).is_some_and(Column::is_sortable) {
            return false;
        }
        let direction = match self.sort {
            Some((active, dir)) if active == column => dir.reversed(),
            _ => SortDirection::Ascending,
        };
        self.sort = Some((column, direction));
        self.rows.sort_by(|a, b| {
            let left = a.get(column).map_or("", |c| &**c);
            let right = b.get(column).map_or("", |c| &**c);
            let ordering = left.cmp(right);
            match direction {
                SortDirection::Ascending => ordering,
                SortDirection::Descending => ordering.reverse(),
            }
        });
        true
    }

    #[doc = include_str!("table.doc/sorted_column.md")]
    #[must_use]
    pub fn sorted_column(&self) -> Option<usize> {
        self.sort.map(|(column, _)| column)
    }

    #[doc = include_str!("table.doc/sort_direction.md")]
    #[must_use]
    pub fn sort_direction(&self) -> Option<SortDirection> {
        self.sort.map(|(_, direction)| direction)
    }

    #[doc = include_str!("table.doc/row_count.md")]
    #[must_use]
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    #[doc = include_str!("table.doc/column_count.md")]
    #[must_use]
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    #[doc = include_str!("table.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty() && self.rows.is_empty()
    }

    #[doc = include_str!("table.doc/semantic_role.md")]
    #[must_use]
    pub const fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Table
    }

    #[doc = include_str!("table.doc/view.md")]
    #[must_use]
    pub fn view(&self) -> View {
        let sort_col = self
            .sorted_column()
            .map_or(-1, |i| i64::try_from(i).unwrap_or(-1));

        // The table's parts name themselves in HTML's own vocabulary, under the
        // reserved LOCAL_NAME symbol the matcher reads as an element's *type*
        // (spec §4.13). They used to carry a private `role` property instead — not
        // a class, not a local name, invisible to CSS — so the UA sheet's `table`,
        // `th`, `td` and `tr:hover` rules were dead against this widget while
        // matching the same markup written by hand. A header cell is `th` and a
        // data cell `td`, the distinction the shared `TABLE_CELL_ID` cannot carry
        // and the one the sheet styles differently.
        let header_cells: Vec<ViewNode> = self
            .columns
            .iter()
            .map(|column| {
                ViewNode::element(Self::TABLE_CELL_ID)
                    .with_prop(ViewProperty::new(LOCAL_NAME, ViewValue::from("th")))
                    .with_prop(ViewProperty::new(
                        Self::PROP_SORTABLE,
                        ViewValue::Bool(column.sortable),
                    ))
                    .with_children(ViewFragment::from_nodes(vec![
                        Text::new(column.title.clone()).into_view_node(),
                    ]))
            })
            .collect();
        let header = ViewNode::element(Self::TABLE_ROW_ID)
            .with_prop(ViewProperty::new(LOCAL_NAME, ViewValue::from("tr")))
            .with_children(ViewFragment::from_nodes(header_cells));

        let mut rows: Vec<ViewNode> = Vec::with_capacity(self.rows.len() + 1);
        rows.push(header);
        for row in &self.rows {
            let cells: Vec<ViewNode> = row
                .iter()
                .map(|cell| {
                    ViewNode::element(Self::TABLE_CELL_ID)
                        .with_prop(ViewProperty::new(LOCAL_NAME, ViewValue::from("td")))
                        .with_children(ViewFragment::from_nodes(vec![
                            Text::new(cell.clone()).into_view_node(),
                        ]))
                })
                .collect();
            rows.push(
                ViewNode::element(Self::TABLE_ROW_ID)
                    .with_prop(ViewProperty::new(LOCAL_NAME, ViewValue::from("tr")))
                    .with_children(ViewFragment::from_nodes(cells)),
            );
        }

        let node = ViewNode::element(Self::COMPONENT_ID)
            .with_prop(ViewProperty::new(LOCAL_NAME, ViewValue::from("table")))
            .with_prop(ViewProperty::new(
                Self::PROP_SORT_COL,
                ViewValue::Int(sort_col),
            ))
            .with_children(ViewFragment::from_nodes(rows));
        View::single(node)
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
        assert_eq!(&*t.rows[0][0], "Alice");
        assert_eq!(&*t.rows[2][0], "Charlie");
    }

    #[test]
    fn sort_again_flips_direction() {
        let mut t = sample();
        assert!(t.sort_by(0));
        assert!(t.sort_by(0));
        assert_eq!(t.sort_direction(), Some(SortDirection::Descending));
        assert_eq!(&*t.rows[0][0], "Charlie");
        assert_eq!(&*t.rows[2][0], "Alice");
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
