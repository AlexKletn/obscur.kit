The crate's hand-rolled CSS tokenizer and the token model it emits.

[`Tokenizer`] is the lexical layer under every parser in this module: a
simplified CSS Syntax Level 3 scanner (spec §4.13) whose whole interface is
one call — text in, `(tokens, errors)` out — over ~250 lines of scanning
rules. [`CssToken`] and [`CssTokenKind`] are its output alphabet and have no
meaning apart from it, so they live here with it, the way
[`super::selector_parser`] keeps its parsed selector types beside the parser
that produces them.

Everything here is `pub(crate)`: [`super::stylesheet_parser`] and the sibling
parsers ([`super::selector_parser`], [`super::value_parser`], and friends)
all consume this one token stream, so a stylesheet is tokenized exactly once.
