Splits a color function's argument list into its color components and an
optional alpha, tolerating both the legacy comma syntax and the modern
space-plus-`/` syntax (spec CSS Color 4 §4). Commas separate components;
a single `/` delimiter introduces the alpha. Returns [`None`] on an
unrecognised token or a malformed (double-`/`) list.
