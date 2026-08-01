**SPECIFICATION-DERIVED** grid-layout cases for obscur.kit §4.1 / §4.13.

Every expected number below was produced by an agent reading the
specification — obscur.kit §4 and the W3C css-grid-2 text it normatively
cites — with **no access to this repository's source**. A failure here is a
defect report about the engine, and must not be "corrected" by editing the
expected value.

Two translation notes, both recorded rather than hidden:

* The deriver's `column-gap` is authored here as the `gap` shorthand, because
  `column-gap` / `row-gap` are not modelled as properties in this workspace.
  Every gap assertion in this file reads only inline-axis geometry, which the
  shorthand sets identically.
* "sized under a max-content constraint" / "available inline space is
  indefinite" is expressed by making the grid a `flex: 0 0 auto` item of a flex
  row, which is the shrink-to-fit context available through the public API.
