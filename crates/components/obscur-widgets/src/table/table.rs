//! `Table`: a headless columns/rows data table with single-column sort
//! (spec §2.6).

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Text;
use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// A single column definition in a [`Table`].
///
/// Each column carries a display `title` and whether it is
/// [`sortable`](Self::is_sortable). Non-sortable columns are ignored by
/// [`Table::sort_by`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    title: Box<str>,
    sortable: bool,
}

impl Column {
    /// Creates a non-sortable column showing `title`.
    #[must_use]
    pub fn new(title: impl Into<Box<str>>) -> Self {
        Self {
            title: title.into(),
            sortable: false,
        }
    }

    /// Sets whether the column is sortable, returning it (fluent builder).
    #[must_use]
    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    /// Returns the column title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns whether the column is sortable.
    #[must_use]
    pub const fn is_sortable(&self) -> bool {
        self.sortable
    }
}

/// The direction of a [`Table`]'s active sort.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Ascending order (A→Z).
    Ascending,
    /// Descending order (Z→A).
    Descending,
}

impl SortDirection {
    /// Returns the opposite direction.
    #[must_use]
    pub const fn reversed(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

/// A headless columns/rows data table with single-column sort (spec §2.6).
///
/// `Table` owns its [`Column`] definitions, its `rows` (each row is a [`Vec`] of
/// cell strings, one per column), and the active [`sort`](Self::sorted_column)
/// (a column index plus a [`SortDirection`], if any). It bakes in **no** visual
/// theme: [`view`](Self::view) derives a [`SemanticRole::Table`] element carrying
/// the sorted column index, a header [`SemanticRole::Row`] child (one
/// [`SemanticRole::Cell`] per column, carrying its title and sortable flag), and
/// one row child per data row (one cell child per cell), rendered by the style
/// layer keyed on [`COMPONENT_ID`](Self::COMPONENT_ID).
///
/// The state machine is [`sort_by`](Self::sort_by): sorting by a sortable column
/// stable-sorts the rows by that column's cell text ascending; sorting by the
/// same column again flips the direction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    columns: Vec<Column>,
    rows: Vec<Vec<Box<str>>>,
    sort: Option<(usize, SortDirection)>,
}

impl Table {
    /// Stable component id for the table widget (see [`Button::COMPONENT_ID`] for
    /// the shared `0x00D0_00NN` scheme).
    ///
    /// [`Button::COMPONENT_ID`]: crate::button::Button::COMPONENT_ID
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_000A);

    /// Stable component id shared by header and data row child elements.
    const TABLE_ROW_ID: ComponentId = ComponentId::from_raw(0x00D0_001A);
    /// Stable component id shared by header and data cell child elements.
    const TABLE_CELL_ID: ComponentId = ComponentId::from_raw(0x00D0_002A);

    /// Interned name of the semantic-role property (`role`); shares the
    /// `0x00D1_NNXX` scheme with `NN = 0A` (table): role=`00`, sort-col=`01`,
    /// sortable=`02` (on header cells).
    const PROP_ROLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0A00);
    /// Interned name of the sorted-column-index property (`-1` = unsorted).
    const PROP_SORT_COL: InternedSymbol = InternedSymbol::from_raw(0x00D1_0A01);
    /// Interned name of the per-header-cell `sortable` state property.
    const PROP_SORTABLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0A02);

    /// Creates an empty, unsorted table.
    #[must_use]
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            sort: None,
        }
    }

    /// Appends a single column, returning the updated table (fluent builder).
    #[must_use]
    pub fn column(mut self, column: Column) -> Self {
        self.columns.push(column);
        self
    }

    /// Appends every column from `columns`, returning the updated table.
    #[must_use]
    pub fn with_columns<I>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = Column>,
    {
        self.columns.extend(columns);
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
        self.rows.push(cells.into_iter().map(Into::into).collect());
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
            self.rows.push(row.into_iter().map(Into::into).collect());
        }
        self
    }

    /// Sorts the rows by the cell text of column `column`, returning `true` on
    /// success. Sorting by a fresh column sorts ascending; sorting by the
    /// already-active column flips the direction. The sort is **stable**.
    /// Returns `false` (leaving state untouched) when `column` is out of range or
    /// its column is not [`sortable`](Column::is_sortable).
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

    /// Returns the index of the sorted column, if any.
    #[must_use]
    pub fn sorted_column(&self) -> Option<usize> {
        self.sort.map(|(column, _)| column)
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

    /// Derives the semantic view: a [`COMPONENT_ID`] element carrying the role
    /// and sorted-column index (`-1` when unsorted), a header row child (one cell
    /// per column, each carrying its title and sortable flag), then one row child
    /// per data row (one cell child per cell).
    ///
    /// [`COMPONENT_ID`]: Self::COMPONENT_ID
    #[must_use]
    pub fn view(&self) -> View {
        let sort_col = self
            .sorted_column()
            .map_or(-1, |i| i64::try_from(i).unwrap_or(-1));

        let header_cells: Vec<ViewNode> = self
            .columns
            .iter()
            .map(|column| {
                ViewNode::element(Self::TABLE_CELL_ID)
                    .with_prop(ViewProperty::new(Self::PROP_ROLE, ViewValue::from("cell")))
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
            .with_prop(ViewProperty::new(Self::PROP_ROLE, ViewValue::from("row")))
            .with_children(ViewFragment::from_nodes(header_cells));

        let mut rows: Vec<ViewNode> = Vec::with_capacity(self.rows.len() + 1);
        rows.push(header);
        for row in &self.rows {
            let cells: Vec<ViewNode> = row
                .iter()
                .map(|cell| {
                    ViewNode::element(Self::TABLE_CELL_ID)
                        .with_prop(ViewProperty::new(Self::PROP_ROLE, ViewValue::from("cell")))
                        .with_children(ViewFragment::from_nodes(vec![
                            Text::new(cell.clone()).into_view_node(),
                        ]))
                })
                .collect();
            rows.push(
                ViewNode::element(Self::TABLE_ROW_ID)
                    .with_prop(ViewProperty::new(Self::PROP_ROLE, ViewValue::from("row")))
                    .with_children(ViewFragment::from_nodes(cells)),
            );
        }

        let node = ViewNode::element(Self::COMPONENT_ID)
            .with_prop(ViewProperty::new(Self::PROP_ROLE, ViewValue::from("table")))
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
