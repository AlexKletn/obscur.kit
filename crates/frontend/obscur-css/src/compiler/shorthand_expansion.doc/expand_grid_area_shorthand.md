Expands the `grid-area` shorthand into its FOUR line longhands (spec §4.13).

The value is split on `/` into up to four `<grid-line>` edges in CSS order —
`row-start / column-start / row-end / column-end` — mapping to
[`PropertyId::GridRowStart`], [`PropertyId::GridColumnStart`],
[`PropertyId::GridRowEnd`], [`PropertyId::GridColumnEnd`]. Omitted edges follow
the CSS filling rules over the LINE-based forms only:

* a lone `<custom-ident>` (`grid-area: header`) is a **named-area** placement:
  it is carried as a single [`PropertyId::GridArea`] marker the runtime
  resolves against the item's container `grid-template-areas`, not expanded to
  the four line longhands.
* `a` (a lone line number) → all four edges are `a`.
* `a / b` → `row-start=a, column-start=b`, both ends `auto`.
* `a / b / c` → `row-start=a, column-start=b, row-end=c`, `column-end=auto`.
* `a / b / c / d` → the four edges directly.

An unparseable edge, or more than four values, degrades the whole shorthand to
a single [`CompiledDeclaration::Unsupported`] rather than dropping it.
