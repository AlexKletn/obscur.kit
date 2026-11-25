#[doc = include_str!("../../docs/api/parser/css_tokenizer.doc/CssTokenKind.md")]
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CssTokenKind {
    /// An identifier such as `flex`, `--custom`, or `-webkit-box`.
    Ident(Box<str>),
    /// An at-keyword such as `@media`; stores `media` (without the `@`).
    AtKeyword(Box<str>),
    /// A hash token such as `#id` or `#ff0000`; stores the name (without `#`).
    Hash(Box<str>),
    /// A quoted string; stores the inner text (without the quotes).
    Str(Box<str>),
    /// A unitless number such as `12` or `-3.5`.
    Number(f64),
    /// A percentage such as `50%`; stores the numeric part (`50`).
    Percentage(f64),
    /// A dimension such as `12px`; stores the number and the unit text.
    Dimension {
        /// The numeric part of the dimension.
        value: f64,
        /// The unit text (for example `px`, `rem`, `deg`).
        unit: Box<str>,
    },
    /// A single delimiter or operator character (`> + ~ * = | ^ $ . ! / &` …).
    Delim(char),
    /// A `:` colon.
    Colon,
    /// A `;` semicolon.
    Semicolon,
    /// A `,` comma.
    Comma,
    /// A `{` left brace.
    LBrace,
    /// A `}` right brace.
    RBrace,
    /// A `(` left parenthesis.
    LParen,
    /// A `)` right parenthesis.
    RParen,
    /// A `[` left bracket.
    LBracket,
    /// A `]` right bracket.
    RBracket,
    /// A maximal run of collapsible whitespace.
    Whitespace,
}
