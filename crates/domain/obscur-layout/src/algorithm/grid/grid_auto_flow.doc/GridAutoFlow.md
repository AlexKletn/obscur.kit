How a grid container auto-places its fully-auto items (`grid-auto-flow`).

The flow mode has two independent facets, folded here into a small `Copy`
enum:

* **axis** — [`Row`](Self::Row)/[`RowDense`](Self::RowDense) fill *row-major*
  (across each row, then wrap to the next row), while
  [`Column`](Self::Column)/[`ColumnDense`](Self::ColumnDense) fill
  *column-major* (down each column, then wrap to the next column).
* **packing** — the plain [`Row`]/[`Column`] modes pack *sparsely*: an
  auto-placement cursor only ever moves forward, so a hole left behind (by a
  definite item or a span that wrapped) is never backfilled. The
  `*Dense` modes pack *densely*: every auto item is searched from the grid
  origin, so a smaller later item backfills an earlier hole.

[`Default`] is [`Row`](Self::Row) — the CSS initial value and the engine's
historical row-major, non-backfilling behaviour — so a container that does
not author `grid-auto-flow` auto-places exactly as before.
