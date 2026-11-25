#[doc = include_str!("../../docs/api/table/table.doc/Column.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub(super) title: Box<str>,
    pub(super) sortable: bool,
    pub(super) width: Option<u32>,
}

impl Column {
    /// Creates a non-sortable column showing `title`.
    #[must_use]
    pub fn new(title: impl Into<Box<str>>) -> Self {
        Self {
            title: title.into(),
            sortable: false,
            width: None,
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
