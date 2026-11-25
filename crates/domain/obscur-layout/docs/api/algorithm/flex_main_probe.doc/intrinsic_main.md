The item's main-axis size for an **intrinsic** `flex-basis` / `width` /
`height` keyword, or [`None`] when `dim` is not one (CSS Sizing 3 §4.1).

The flex twin of [`intrinsic_inline_size`](super::intrinsic_inline_size), and
the same three answers — but flex asks them of ONE item rather than of a run of
stacked children, and asks along whichever axis is the main one, so the probe
is a measure of that item instead of a pass over children:

* `min-content` — the item offered no main-axis space;
* `max-content` — the item offered unbounded main-axis space;
* `fit-content` — `min(max-content, max(min-content, stretch))`, where
  `stretch` is the container's main extent (CSS 2.1 §10.3.5's shrink-to-fit).

Without this the keywords fell through [`Dimension::resolve`]'s [`None`] into
the measured-natural-size fallback — i.e. `flex-basis: max-content` silently
meant `flex-basis: auto`.

# Cost

One extra measure per intrinsic item (`fit-content` two), and only for an item
that authored a keyword: [`Dimension::resolve`]-shaped values return here
immediately without measuring.
