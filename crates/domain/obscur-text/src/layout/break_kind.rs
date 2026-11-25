#[doc = include_str!("../../docs/api/layout/line_break.doc/BreakKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BreakKind {
    /// A permitted (optional) wrap point, e.g. after a space.
    Soft,
    /// A mandatory break, e.g. a line feed.
    Hard,
}
