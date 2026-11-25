A font-relative `ch` length: this many times the advance of the `0`
(ZERO, U+0030) glyph in the element's own font
([`LengthContext::ch_base`]).

Unlike `em`, this depends on the *face*, not just the size: it is how a
column is sized by character count (`width: 60ch`), which is exact in a
monospace face and approximate in a proportional one. See
[`LengthContext::ch_base`] for what happens when the glyph cannot be
measured.
