Lowers the argument list of a `url(...)` component into its raw URL/path
string, or [`None`] when it is empty.

A quoted `url("foo.png")` arrives as a single [`ParsedValue::String`] whose
text is used verbatim. An unquoted `url(bar.png)` arrives as a run of
components (the tokenizer has no dedicated URL token), so its body is
reconstructed by concatenating each component's textual form in author order
(`bar` + `.` + `png` = `bar.png`). Leading/trailing whitespace was already
dropped by the value parser. An empty `url()` yields [`None`].
