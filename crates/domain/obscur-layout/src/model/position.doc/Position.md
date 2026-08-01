How a node is positioned relative to the normal flow (spec §4.1; CSS
`position`).

All four CSS positioning schemes are modelled and honoured by the layout
coordinator:

* [`Static`](Self::Static) — the default. The node is placed by its
  formatting context and its insets are ignored.
* [`Relative`](Self::Relative) — the node stays **in flow** (its flow box and
  its siblings are unchanged), then its final position (and its whole
  subtree) is offset by its resolved insets.
* [`Absolute`](Self::Absolute) — **out of flow**: the node does not
  participate in its parent's formatting context (it neither affects nor is
  affected by in-flow siblings) and is positioned by its insets within the
  padding box of its nearest positioned ancestor (its *containing block*).
* [`Fixed`](Self::Fixed) — out of flow like [`Absolute`](Self::Absolute), but
  its containing block is always the viewport, ignoring ancestors.
* [`Sticky`](Self::Sticky) — a **hybrid**. The node stays **in flow** exactly
  like [`Relative`](Self::Relative) (its flow box and its siblings are
  unchanged), but its *used* position is then clamped so that, once its scroll
  container has scrolled past its inset threshold, the node sticks to the
  scrollport edge — never moving above its static position and never past its
  containing block. With a zero scroll offset (or before the threshold is
  reached) its used position equals its static position, so it is transparent.
  The single scroll container is currently the **viewport**; per-container
  overflow scroll containers are a follow-up.

[`Relative`](Self::Relative), [`Absolute`](Self::Absolute),
[`Fixed`](Self::Fixed), and [`Sticky`](Self::Sticky) each *establish a
containing block* for their positioned descendants
([`is_positioned`](Self::is_positioned)); only [`Absolute`](Self::Absolute) and
[`Fixed`](Self::Fixed) are *out of flow* ([`is_out_of_flow`](Self::is_out_of_flow)).
[`Sticky`](Self::Sticky), like [`Relative`](Self::Relative), is **in flow**.
