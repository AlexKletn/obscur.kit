#[doc = include_str!("../../docs/api/parser/value_parser.doc/ParsedValue.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedValue {
    /// A bare identifier keyword such as `flex`, `auto`, or `inherit`.
    Keyword(Box<str>),
    /// A unitless number such as `12` or `-0.5`.
    Number(f64),
    /// A percentage such as `50%`; stores the numeric part (`50`).
    Percentage(f64),
    /// A dimension such as `12px` or `1.5rem`; the unit is kept verbatim and is
    /// validated later by the compiler.
    Dimension {
        /// The numeric part of the dimension.
        value: f64,
        /// The unit text (for example `px`, `rem`, `fr`, `deg`, `ms`).
        unit: Box<str>,
    },
    /// A quoted string (without its quotes).
    String(Box<str>),
    /// A hash value such as a hex colour `#ff0000`; stored without the `#`.
    Hash(Box<str>),
    #[doc = include_str!("../../docs/api/parser/value_parser.doc/Function.md")]
    Function {
        /// The function name (empty for a bare parenthesised group).
        name: Box<str>,
        /// The nested argument component list.
        args: Vec<ParsedValue>,
    },
    /// A `,` comma separator between components (for example between font
    /// families or `transition` layers).
    Comma,
    /// A bare operator/delimiter character (for example `+`, `-`, `*`, `/`
    /// inside `calc()`).
    Delimiter(char),
    /// A bracketed line-name group `[ a b … ]` from a grid track list
    /// (`grid-template-columns: [sidebar-start] 200px [main]`). The names are the
    /// `<custom-ident>`s inside the brackets, in author order; an empty group
    /// (`[]`) carries no names. Only the grid track-list compiler reads this; any
    /// other property degrades to an unsupported-feature diagnostic (spec §4.13).
    LineNames(Box<[Box<str>]>),
}
