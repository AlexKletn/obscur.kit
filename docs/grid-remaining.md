# CSS Grid — remaining minor items

CSS Grid in obscur.kit_v2 is practically complete and verified end-to-end (see
`crates/facade/obscur-kit/tests/grid_css.rs` and `grid_placement_css.rs`). What
is implemented: track sizing (fixed / `fr` / `auto` / `minmax` / `%`),
`repeat()` (numeric + `auto-fill`/`auto-fit`), implicit tracks
(`grid-auto-columns`/`grid-auto-rows`), auto-placement, line-based placement,
spanning, negative lines, named lines (`[name]`), `grid-area` (numeric + named),
named areas (`grid-template-areas`), `grid-auto-flow` (row / column / dense),
item alignment (`justify-items`/`align-items`/`justify-self`/`align-self` +
`place-*`), track distribution (`justify-content`/`align-content` +
`place-content`), and explicit container sizing.

The following are the deliberately-deferred remnants. Each is edge/advanced and
documented at its site in code.

## Alignment
- **Baseline alignment** — `align-items: baseline` / `align-self: baseline` are
  not modelled (no baseline metric threaded from text layout into the grid item
  box). Falls back to `start`.
- **`stretch` track distribution** — `justify-content: stretch` /
  `align-content: stretch` are not modelled: `JustifyContent` has no `Stretch`
  variant, so a `stretch` content-distribution is treated as `start`. (Item-level
  `stretch` — filling the cell — *is* implemented; this is only the *content*
  distribution keyword.)

## Named lines & areas
- **Area-implicit named lines** — a named area `foo` in `grid-template-areas`
  should implicitly create the lines `foo-start` / `foo-end` on both axes;
  currently only areas placed by `grid-area: foo` resolve, not
  `grid-column: foo-start`. Needs an area-name iterator on `GridAreaTemplate`.
- **nth line name** — `grid-column: main 2` (the 2nd line named `main`) resolves
  to the **first** occurrence; the integer disambiguator is ignored.
- **`grid-template-areas: none`** — degrades to an `Unsupported` diagnostic
  rather than clearing areas. Harmless (absent = the same effect), but not the
  canonical no-op.

## Track sizing
- **`fr` in an implicit track** — an `fr` implicit track (from
  `grid-auto-columns/-rows: 1fr`) has no free-space basis of its own, so it
  collapses to its floor (its `min`, else `0`). Explicit `fr` tracks are
  unaffected.
- **`fr`/`auto` inside an `auto-fill`/`auto-fit` pattern** — invalid per CSS;
  contributes `0` to the fitted extent, so the repeat count falls back toward 1.

## Placement
- **Half-definite items in `column` auto-flow** — an item with one definite axis
  under `grid-auto-flow: column` uses the row-oriented flow helper; fully-auto
  column flow is correct, half-definite column-flow edge cases are not specially
  handled.
- **Dense packing vs source order** — first-fit dense backfills holes but does
  not reorder to the strict CSS "most-packed" outcome in every pathological case.

## Not started
- **Subgrid** (`grid-template-columns: subgrid`) — a child grid adopting the
  parent's tracks. A substantial separate feature; not begun.
- **`grid-template` shorthand** (the combined areas + rows + columns form) — only
  the longhands (`grid-template-columns/-rows/-areas`) are lowered.

---

Ranked by likely value if revisited: subgrid > area-implicit named lines >
`stretch`/baseline alignment > the rest.
