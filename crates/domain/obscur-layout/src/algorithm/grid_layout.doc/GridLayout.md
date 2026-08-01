CSS-grid layout with real per-track sizing (spec §4.1, §4.2).

Children flow **row-major** into a grid described by a [`GridTemplate`]:
child `0` fills column `0` of row `0`, child `1` column `1`, and so on,
wrapping to a new row once every column in the current row is filled. Each
column and row is sized independently by its [`TrackSize`] — a fixed pixel
length, a flexible `fr` fraction, a content-sized `auto`, or a
`minmax(min, max)` clamp.

# Configuration

The grid carries a [`GridTemplate`] (the column and row track lists) plus a
horizontal `column_gap` and vertical `row_gap`. The uniform constructor
[`GridLayout::new`] builds an *N equal columns* grid by using `N` `1fr`
columns and all-implicit auto rows, so a plain `new(n)` still lays out `n`
equal-width columns exactly as before.

# Algorithm

1. **Content area** — the container's own explicit `width`/`height` (resolved
   against `constraints.max`) gives the content width/height; an `Auto` axis
   falls back to `constraints.max` minus `padding` (each clamped ≥ 0). Tracks,
   `fr` distribution, and percentages all resolve against this extent.
2. **Column track sizing** — with `n = columns.len()` and
   `available = content_width - (n - 1) * column_gap`, each column resolves to:
   `Fixed(px)` → `px`; `Percent(f)` → `f * content_width` (its basis is the
   grid container's content extent, known only here); `Auto` → the largest
   measured content width among the items auto-placed in it (measured under
   loose constraints); `MinContent` / `MaxContent` → the largest min-content /
   max-content contribution among its items, each measured by offering the item
   no inline space / unbounded inline space (CSS Sizing 3 §4.1) — unlike `Auto`
   these ignore the space the container has and do not stretch into it;
   `Fr(f)` → a share `f / Σfr` of the space left after the
   fixed + percentage + auto tracks; and
   `MinMax(min, max)` → the `max` function resolved, then floored at the `min`
   function (so `minmax(px, fr)` grows flexibly from `px` up). `fr` space is
   distributed exactly like [`FlexLayout`](super::FlexLayout) shares free
   space among grow factors.
3. **Row track sizing** — the same resolution over the explicit `rows` using
   `content_height` and `row_gap`. Rows past the explicit list are *implicit*
   and sized by the template's `grid-auto-rows`
   ([`auto_rows`](GridTemplate::auto_rows)) — cycled by the n-th implicit row
   using index `n % len` — defaulting (empty list) to `Auto`, as tall as the
   tallest item placed in them.
4. **Placement** — each child carries a [`GridPlacement`](super::GridPlacement)
   (its `grid-column` / `grid-row` line pair). Placement runs in two phases:
   *Phase A* honours every item with a definite column and/or row line — a
   fully-definite item lands at its cell, a half-definite item locks the
   definite axis and auto-flows the other — marking each `col-span × row-span`
   block occupied. *Phase B* then places the remaining fully-auto items
   row-major by **first-fit**: each drops into the first free cell (skipping
   occupied cells, wrapping a too-wide span to the next row) that fits its
   span. Definite lines past the template add implicit columns and rows past
   the template are implicit — both sized by `grid-auto-columns` /
   `grid-auto-rows` (cycled), defaulting to `Auto`. With every child left at the default
   all-`Auto` placement this reduces exactly to `(i % n_cols, i / n_cols)`.
5. **Sizing** — an item spanning `s` columns is `Σ(spanned column widths) +
   (s - 1) * column_gap` wide (analogously for a row span with `row_gap`); a
   spanning item attributes its content size to its first spanned track. It is
   measured with its width fixed to that span; a single content-sized (`Auto`
   / implicit) row keeps its own measured height, otherwise it fills its
   spanned row tracks. It is placed at
   `(padding.left + Σ(prior column widths) + start_col * column_gap,
     padding.top  + Σ(prior row heights)  + start_row * row_gap)`.
6. **Border box** — an explicit container axis sizes to that content extent
   plus `padding`; an `Auto` axis sizes to the bounding box of the placed
   children plus `padding`. The result is clamped into `constraints`.

# Out of scope (later steps)

This does real track sizing plus line-based placement and spanning (including
negative, from-end grid lines resolved against the track count), but is still a
subset: no *dense* auto-placement packing (placement is first-fit, never
backfilling earlier holes), no named grid lines, and no
alignment of an item within its track. Half-definite items always auto-flow
the missing axis. Those belong to later work; the CSS parsing and runtime
wiring of placement is a later step.
