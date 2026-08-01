css-values-4 §6.1.1: "Represents the typical advance measure of European
alphanumeric characters, and measured as the used advance measure of the '0'
(ZERO, U+0030) glyph in the font used to render it." and "In the cases where it
is impossible or impractical to determine the measure of the '0' glyph, it must
be assumed to be 0.5em wide by 1em tall."

Case (b) — "a font with no '0' glyph available" — is expressed by installing a
[`MockFontResolver`] with nothing registered, so no face resolves and the `0`
advance is unmeasurable.
