The top-level [`StylesheetParser`] entry point and the rule AST it produces.

This file owns the resilient top-level rule reader (spec §4.13): the loop
that turns a [`CssSource`] into a [`ParsedStylesheet`] by recognising
at-rules, selector-list preludes and declaration blocks, and by recovering to
the next rule boundary when one of them is malformed. The lexical layer it
runs on lives beside it — [`super::css_tokenizer`] scans the text into tokens
(once per stylesheet, for every parser in this module) and
[`super::token_cursor`] is how the sub-parsers read a slice of them back.
