#[doc = include_str!("../../docs/api/route/route_pattern.doc/PatternSegment.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternSegment {
    /// A literal segment that must equal the corresponding path segment.
    Static(Box<str>),
    /// A `:name` segment that captures exactly one path segment under `name`.
    Param(Box<str>),
    /// A `*name` segment that captures all remaining path segments (joined by
    /// `/`) under `name` and ends the match.
    Wildcard(Box<str>),
}
