The §4.2 **layout** cache — the twin of [`MeasureCache`](super::measure_cache::MeasureCache), and what makes
`arrange` incremental rather than O(nodes) (spec §3.4 step 8, §4.3 "early exit
when geometry is stable").

It retains, per node, the [`ArrangeKey`] that node was last arranged under, and
the absolute [`LayoutBoxes`] the last completed pass produced. When
[`arrange`](super::arrange_pass::arrange) reaches a node whose key is unchanged **and** whose entry survived
[`evict`](Self::evict), the node's whole subtree is provably identical to last
frame's, so the walk stops there and serves the retained boxes instead of
re-running the subtree's layout algorithms.

# The two conditions, and why neither alone is enough

* **The key alone is not enough.** A node put back in the same place at the
  same size can still have restyled — a `flex-direction` flip re-lays its
  children inside an unchanged box.
* **The dirty set alone is not enough**, and this is the one that would be
  *silently* wrong. `arrange` writes **absolute** positions. A node moving
  shifts every descendant; in a flow a sibling's size change moves every later
  sibling; a flex line re-flows whole when one item resizes — and **none of
  those moved nodes are dirty**. A cache that skipped every non-dirty node
  would leave them painted at last frame's coordinates. What catches them is
  that their `origin` (and `constraints`) is computed *fresh* by their parent's
  real algorithm run and only then compared: the parent is dirty-by-closure, so
  it re-runs, so its children's keys are recomputed, so the moved ones miss.
  Propagation is *performed*, never inferred.

[`arranged`](Self::arranged) / [`arranged_count`](Self::arranged_count) report
what the **last** pass actually re-arranged — the only way to tell a served
subtree from a recomputed one, and what this module's tests assert on.
