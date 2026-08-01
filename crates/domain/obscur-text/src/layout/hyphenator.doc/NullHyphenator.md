A [`Hyphenator`] that never proposes a break point (spec §5.10).

This is the domain default: wrapping through it reproduces the historical
whitespace-only line breaking exactly, so a paragraph laid out with a
`NullHyphenator` is byte-identical to one laid out before hyphenation existed.
A host installs a real adapter to enable in-word breaking.
