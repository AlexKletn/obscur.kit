/// A comparator used in range media/container features.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaComparator {
    /// `=` equality.
    Eq,
    /// `<` strictly less than.
    Lt,
    /// `<=` less than or equal.
    Le,
    /// `>` strictly greater than.
    Gt,
    /// `>=` greater than or equal.
    Ge,
}
