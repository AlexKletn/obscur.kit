Normal-flow / block layout (spec §4.1, §4.2).

Stacks children top-to-bottom in document order, each occupying the full
content width, and sizes the node to fit unless an explicit height is given.

# Rules

* **Content width** — resolves `style.width` against the containing block the
  caller stated ([`BoxConstraints::percent_width`], which falls back to the
  constraint's max width); if `Auto`, uses the available content width
  (`constraints.max.width` minus horizontal border + padding). Clamped to be
  non-negative.
* **Intrinsic width (CSS Sizing 3 §4.1)** — `min-content` / `max-content` /
  `fit-content` are answered by *measuring* the children rather than by
  arithmetic: the children are re-measured under a no-space (`min-content`) or
  unbounded (`max-content`) inline offer, and the widest wins. See
  [`intrinsic_inline_size`], which also explains why these cost an extra
  measuring pass. In the **block** axis the two intrinsic sizes coincide with
  the content height, so `height: max-content` is `height: auto` for a block
  and needs no separate path.
* **Intrinsic min/max bounds** — `min-width` / `max-width` take the same
  keywords, and are answered by the same probe as an intrinsic `width`, so
  `min-width: min-content` floors the box at its widest unbreakable run
  whatever its `width` resolved to. Their block-axis twins ride on the
  coincidence above: an intrinsic `min-height` / `max-height` is the height the
  children stacked to, at no extra measuring cost.
* **Child stacking** — each child is measured with
  [`BoxConstraints::loose`] up to `(content_width − its inline margins,
  remaining_height)`, placed at the content-box left edge **plus its
  `margin-left`**, with its top at the running `y` offset. When this box's own
  block size is definite, that offer carries this box's **content box** as the
  child's percentage basis — a separate quantity from the offer, see below.
* **Inline margins (CSS 2.1 §10.3.3)** — a child's *fixed* `margin-left` /
  `margin-right` take from the inline space it is offered, so a `width: auto`
  child fills what is left, and `margin-left` offsets it. An over-constrained
  box (definite width + both margins) drops its *end* margin — the one on the
  inline-end side of `style.direction`: `margin-right` under `ltr` (the child
  stays line-left), `margin-left` under `rtl` (the child settles line-right).
* **`auto` inline margins (CSS 2.1 §10.3.3)** — an `auto` edge reserves
  nothing; it *absorbs* the space left over once the child's used width is
  known. Both edges `auto` splits the leftover evenly — `margin: 0 auto`
  centring; one edge `auto` takes it all, pushing the box to the other side.
  A `width: auto` child needs no special case: it already filled what it was
  offered, so the leftover is zero and its `auto` margins resolve to zero,
  exactly as §10.3.3 says. In the block axis `auto` is always zero
  (§10.6.3): a container's height *is* its content, so there is no leftover
  to absorb.
* **Block margins + vertical margin collapsing (CSS 2.1 §8.3.1)** — each
  in-flow child's block-axis (`margin.top` / `margin.bottom`) margins take
  effect, and the bottom margin of one block collapses with the top margin
  of the next into a single used margin. The used value of a collapsing set
  is the **`{max_positive, min_negative}`** reduction: adjoin a margin by
  keeping the largest positive and the most-negative seen; solve the set as
  `max_positive + min_negative` (so two positives take the larger, two
  negatives take the more-negative, and a mixed pair sums — `+30` with `−20`
  → `+10`). The separation placed between two siblings is
  `gap + collapsed_margin`: `row-gap` does **not** collapse and is additive
  (Box Alignment), so with zero child margins the separation is exactly
  `gap` — byte-identical to a gap-only stack. An empty, zero-height child
  with no top/bottom border or padding has its own top and bottom margins
  *adjoining*, so it **collapses through**: the open collapsing set spans
  across it and a run of such empties between two siblings collapses into a
  single margin.
* **Content height** — resolves `style.height` if set — against the containing
  block ([`BoxConstraints::percent_height`]), not against the space this box was
  offered — otherwise the last child's collapsed bottom edge (the trailing
  collapsing set is contained, see below). Clamped to be non-negative.
* **Border box** — content size plus border and padding, then clamped into
  the incoming `constraints`.

Child boxes are positioned relative to this node's origin, already offset into
the content box by border + padding.

# Deferred — the remaining §8.3.1 piece

**Parent–child / root collapse-through across the container boundary** (a
first child's top margin or a last child's bottom margin protruding *out* of
this box to collapse with the parent's own margin) is **not** modelled: it
requires the [`LayoutOutput`] / [`LayoutAlgorithm`] contract to carry a
block's *adjoining top/bottom margins* up to the parent formatting context —
a protocol change touching every algorithm. For now the first child's top
margin and the last child's bottom margin are **contained** within this box's
content (no protrusion). Also out of scope: BFC-root collapse suppression
nuances and clearance. Floats and inline formatting / line boxes are likewise
not modelled. A **vertical** `writing-mode` rotates the block axis to
horizontal — that path is [`layout_vertical`], and the rules above describe the
default `horizontal-tb` mode.

# Percentage margins resolve against the *width*, on every edge

A `%` margin — including `margin-top` / `margin-bottom` — resolves against this
container's `content_width` (CSS 2.1 §8.3). Reading that as a typo is the
natural reaction; it is deliberate. A block's height usually depends on its
content, so resolving a block-axis percentage against it would be circular. CSS
breaks the cycle by picking one axis, and the inline one is the axis that is
definite by the time children are laid out.

# A percentage *size* resolves against the containing block, not the space left

The offer a child is measured under and the basis its percentages resolve
against are two different quantities, and this algorithm used to have only one.
Each child was offered the block extent *remaining* below its predecessors and
resolved `height: 50%` against that offer, so in a 240px-tall container the
first child was right (120) and every sibling after it compounded the error —
the second `50%` took half of the 120 that remained (60), the second `30%` of
two came back 50, `20%`/`80%` came back 48/154, and a second `height: 100%`
computed to **zero** and disappeared with nothing to say why.

css-sizing-3.txt:1027 is unambiguous, and it also settles what should happen
instead of the clamping:

> Otherwise, the percentage is resolved against the containing block's size.
> (The containing block's size is not re-resolved based on the resulting size of
> the box; the contents might thus **overflow** or underflow the containing
> block.)

So the remaining extent keeps its job — the room left for the next sibling, and
the space an `auto`/stretch size fills — while the containing block's content box
travels alongside it as [`BoxConstraints::percent_basis`]. Two children at
`height: 100%` are now 240 each and the second one starts at the container's
bottom edge; the container stays 240 and the overflow is the specified outcome,
not a defect to clamp away.

The basis is attached **always**, and an indefinite block size is stated as
[`f32::INFINITY`] rather than left unset. Leaving it unset was not neutral: a
percentage then fell back to resolving against the space *offered*, so two
`height: 50%` siblings under an `auto`-height parent compounded (120 then 60).
css-sizing-3 §3.2.1 (`specs/w3c/css-sizing-3.txt:554`) says a block percentage
height against an indefinite size **behaves as auto**, and an infinite basis is
how that is spelled through the existing door: [`Dimension::resolve`] already
answers [`None`] for a percentage against a non-finite base, and every caller
already treats that `None` as `auto`.

[`Dimension::resolve`]: crate::model::Dimension::resolve
