The [`PropertyParser`] and its parsed declaration AST (spec §4.13/§4.14).

Parses a single declaration `name: value !important?`. Custom properties
(`--foo`) are accepted with their name kept verbatim. The value is delegated
to [`super::value_parser::ValueParser`]; a trailing `!important` (case
insensitive, with optional whitespace) is recognised and stripped.
