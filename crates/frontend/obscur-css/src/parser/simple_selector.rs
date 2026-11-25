use super::AttrOp;

/// A single simple selector.
#[derive(Debug, Clone, PartialEq)]
pub enum SimpleSelector {
    /// A type/element selector such as `div`.
    Type(Box<str>),
    /// The universal selector `*`.
    Universal,
    /// A class selector such as `.button`.
    Class(Box<str>),
    /// An id selector such as `#main`.
    Id(Box<str>),
    /// An attribute selector such as `[data-x]` or `[type="text"]`.
    Attribute {
        /// The attribute name.
        name: Box<str>,
        /// The match operator, or [`None`] for a bare presence test.
        op: Option<AttrOp>,
        /// The compared value, or [`None`] for a bare presence test.
        value: Option<Box<str>>,
    },
    /// A pseudo-class such as `:hover` or `:nth-child(2n+1)`.
    PseudoClass {
        /// The pseudo-class name (without the leading `:`).
        name: Box<str>,
        /// The raw argument text for a functional pseudo-class, if any.
        arg: Option<Box<str>>,
    },
    /// A pseudo-element such as `::before` (without the leading `::`).
    PseudoElement(Box<str>),
    /// The `::part(name)` exported-part pseudo-element; stores the part name
    /// (§2.12 / §4.15).
    Part(Box<str>),
}
