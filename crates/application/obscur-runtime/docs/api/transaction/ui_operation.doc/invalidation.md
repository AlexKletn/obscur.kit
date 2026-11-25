States this operation as the [`Invalidation`] it causes — the runtime-level
change a frame must classify (spec §3.4 phase 5, §4.3).

This is the seam that gives the operation vocabulary an effect. Route the
result through an
[`InvalidationRouter`](crate::invalidation::InvalidationRouter) and its
phase flags say what the frame recalculates.

The mapping is total and each arm says what the operation *claims*:

* [`ReplaceRoot`](Self::ReplaceRoot) — a **global**
  [`TreeChanged`](InvalidationReason::TreeChanged). It names no node
  because it targets none: the whole tree reconciles.
* [`Rerender`](Self::Rerender) — `TreeChanged` **at that node**. A
  component's output can change its subtree's structure, so this is the
  same cause, narrowed.
* [`InvalidateStyle`](Self::InvalidateStyle) —
  [`StyleChanged`](InvalidationReason::StyleChanged): the node restyles,
  and layout follows because a style can feed geometry.
* [`InvalidateLayout`](Self::InvalidateLayout) —
  [`LayoutChanged`](InvalidationReason::LayoutChanged): geometry only. It
  deliberately does **not** dirty the style phase, which is the entire
  difference from `InvalidateStyle` and the reason the two are separate
  operations.
