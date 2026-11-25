A cheap cursor over one [`RetainedNode`] that implements
[`MatchElement`](obscur_style::selector::MatchElement), the seam the
`obscur-style` selector matcher walks.

A `TreeElement` is a shared reference to the [`RetainedTree`] plus the
[`NodeId`] it points at, so it is `Copy` and allocation-free: navigation
methods ([`parent`](MatchElement::parent),
[`previous_sibling_element`](MatchElement::previous_sibling_element)) mint
new cursors over the same borrowed tree rather than cloning node data. This
is the integrating adapter that lets authored stylesheets match against the
retained tree without `obscur-style` depending on `obscur-tree`.

# First-iteration limitations

- **Attributes** — the retained tree only preserves an element's `class`,
  `id`, and *type* (local name) via
  [`StyleAttributes`](obscur_view::view::StyleAttributes). Consequently
  [`attribute`](MatchElement::attribute) resolves only `"id"` and `"type"`;
  any other attribute selector (e.g. `[data-*]`, `[aria-*]`) matches nothing
  until the tree retains arbitrary attributes.
- **Pseudo-state** — *interaction*-driven states (`:hover`, `:active`,
  `:focus`, …) are sourced from an optional per-node [`PseudoStates`] map the
  runtime owns. A cursor built with [`new`](Self::new) carries no source and
  reports no interaction state (the stateless matching path used by
  `run_matched`); a cursor built with
  [`with_pseudo`](Self::with_pseudo) reads that map, so `:hover`/`:focus`
  finally match once the runtime records the state. Navigation preserves the
  source, so ancestor/sibling combinators over pseudo-classes work too.
  *Structural* states (`:disabled`, `:checked`) are intrinsic to the element:
  they are read from the node's own
  [`StyleAttributes`](obscur_view::view::StyleAttributes) and reported on
  **both** paths, so a `:disabled`/`:checked` rule bites even with no
  interaction map.
- **Top-level siblings** — an element whose logical parent is the synthetic
  [`Root`](obscur_tree::node::NodeKind::Root) grouping node is treated as
  top-level: it reports no [`parent`](MatchElement::parent) (so `:root` and
  child/descendant combinators behave) and is treated as the sole member of
  its sibling list (index `0`, count `1`).
