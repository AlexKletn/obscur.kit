/// The value-comparison operator of an attribute selector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrMatch {
    /// `[a=b]` — the value equals the operand exactly.
    Exact(Box<str>),
    /// `[a~=b]` — the value is a whitespace-separated list containing the operand.
    Includes(Box<str>),
    /// `[a|=b]` — the value equals the operand or begins with `operand-`.
    DashMatch(Box<str>),
    /// `[a^=b]` — the value begins with the operand.
    Prefix(Box<str>),
    /// `[a$=b]` — the value ends with the operand.
    Suffix(Box<str>),
    /// `[a*=b]` — the value contains the operand as a substring.
    Substring(Box<str>),
}
