**SPECIFICATION-DERIVED** distributed-alignment and gutter cases for
obscur.kit §4.1 ("alignment", "gap").

Every expected number below was produced by an agent reading the
specification — obscur.kit §4 and the W3C css-align-3 text it normatively
cites — with **no access to this repository's source**. A failure here is a
defect report about the engine, and must not be "corrected" by editing the
expected value.

Translation note: the deriver's `column-gap` is authored here as the `gap`
shorthand, because `column-gap` / `row-gap` are not modelled as properties in
this workspace. Every assertion in this file reads only main-axis geometry of
a single-line row flex container, which the shorthand sets identically. The
`gap: normal` case inherits a caveat from that: if `normal` fails to parse,
`gap` falls back to its initial value and the test can pass for the wrong
reason — see the test's own comment.
