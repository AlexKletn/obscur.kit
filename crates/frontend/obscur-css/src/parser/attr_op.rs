/// An attribute selector match operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrOp {
    /// `=` exact match.
    Exact,
    /// `~=` whitespace-separated list contains.
    Includes,
    /// `|=` exact or prefix followed by `-`.
    DashMatch,
    /// `^=` prefix match.
    Prefix,
    /// `$=` suffix match.
    Suffix,
    /// `*=` substring match.
    Substring,
}
