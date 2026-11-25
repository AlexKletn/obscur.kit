# CSS Grid — remaining minor items

CSS Grid in obscur.kit_v2 is practically complete and verified end-to-end (see
`crates/facade/obscur-kit/tests/grid_css.rs` and `grid_placement_css.rs`). What
is implemented: track sizing (fixed / `fr` / `auto` / `minmax` / `%` / `fit-content()`),
`repeat()` (numeric + `auto-fill`/`auto-fit`), implicit tracks
(`grid-auto-columns`/`grid-auto-rows`), auto-placement, line-based placement,
spanning, negative lines, named lines (`[name]`), `grid-area` (numeric + named),
named areas (`grid-template-areas`), `grid-auto-flow` (row / column / dense),
the `grid-template` and `grid` shorthands,
item alignment (`justify-items`/`align-items`/`justify-self`/`align-self` +
`place-*`), track distribution (`justify-content`/`align-content` +
`place-content`), and explicit container sizing.

The following are the deliberately-deferred remnants. Each is edge/advanced and
documented at its site in code.

## Alignment
- **Baseline alignment** — `align-items` and `align-self` support `baseline`
  as well as `first baseline` and `last baseline` in the grid block axis.
  These values also work through `place-items` and `place-self`.
  Intrinsic rows account for baseline offsets, and the runtime exports text
  baselines through nested in-flow boxes. Inline-axis sharing remains open.

## Not started
- **Subgrid** (`grid-template-columns: subgrid`) — a child grid adopting the
  parent's tracks. A substantial separate feature; not begun.

---

Ranked by likely value if revisited: subgrid >
baseline alignment > the rest.
