/// A table row retains its insertion key through sorting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TableRow {
    pub(super) key: usize,
    pub(super) cells: Vec<Box<str>>,
}
