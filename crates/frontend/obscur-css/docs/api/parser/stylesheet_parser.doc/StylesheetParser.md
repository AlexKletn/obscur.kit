The top-level CSS parser: turns a [`CssSource`] into a [`ParsedStylesheet`].

The parser is resilient (spec §4.13): a malformed rule is recorded and the
parser recovers to the next rule boundary, so a single mistake never discards
the rest of the stylesheet.
