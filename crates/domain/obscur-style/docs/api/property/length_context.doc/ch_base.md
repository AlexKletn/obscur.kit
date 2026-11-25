The base a `ch` resolves against, in logical pixels.

This is the measured [`ch_width`](Self::ch_width) when one is known. When
it is not, CSS Values 4 §6.1.2 specifies the substitute exactly: *"in the
cases where it is impossible or impractical to determine the measure of
the ZERO glyph, it must be assumed to be 0.5em wide"* — so an unmeasured
`ch` is `0.5 × font-size`.

# Why this is a fallback and not the implementation

`0.5em` is a plausible average for a proportional face and *wrong for a
monospace one*, which is the case `ch` mostly exists to serve — sizing a
column by character count. Resolving every `ch` to `0.5em` would look like
support while silently mis-sizing the one thing anyone writes `ch` for.
The spec permits the substitute only when the measure is unavailable, so
it is reached only when a caller could not measure the glyph, never as a
shortcut around measuring it.
