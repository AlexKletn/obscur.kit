The lexical category of a [`CssToken`] (CSS Syntax Level 3, simplified).

Each variant that carries text stores it already stripped of its delimiters
(a [`Hash`](CssTokenKind::Hash) drops the leading `#`, a
[`Str`](CssTokenKind::Str) drops the surrounding quotes, an
[`AtKeyword`](CssTokenKind::AtKeyword) drops the leading `@`).
