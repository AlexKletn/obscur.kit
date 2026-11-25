/// A media query qualifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaQualifier {
    /// The `not` qualifier.
    Not,
    /// The `only` qualifier.
    Only,
}
