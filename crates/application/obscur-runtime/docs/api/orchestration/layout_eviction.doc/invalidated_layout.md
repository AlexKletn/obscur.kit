The retained layout results `dirty` can have changed, or [`None`] when that
set cannot be bounded and both caches must be dropped (spec §4.3).

# What `measure(N, c)` actually reads

This function is the written-down form of that dependency list, and every
clause below exists because [`measure`](super::measure_pass::measure) reads the thing it names. For a node
`N` under constraints `c`, `measure` reads:

* `styles[M]`, `computed[M]`, `grid_templates[M]` and `tree.get(M)` for every
  `M` in `N`'s subtree — the walk recurses through
  [`children_of`](super::formatting_context::children_of), so **any** dirty node makes every one of its ancestors'
  measures suspect. That is §4.3's *upward* propagation, and it is why the
  ancestor chain of each dirty node is evicted.
* `computed[parent(M)]` for a text `M`, via [`overflow_for_text`](super::measure_pass::overflow_for_text) — the
  container's `text-overflow` is threaded into the text measure. So a dirty
  node's text children are evicted too. That read follows the logical parent
  link. Element children retain their result when their constraints and
  subtree inputs still match.
* `computed[tree.root()]`, via [`root_font_size`](super::root_font_size) — the `rem` base every text
  metric and (through [`length_context`](super::length_resolution::length_context)) every relative length resolves
  against. It is global, so a dirty root cannot be bounded: the answer is
  [`None`].

Nothing else: a node's measure does not read its ancestors' *sizes* (that is
what `c` carries) nor its siblings' anything. Which is the whole point —
§4.3's *downward* rule ("propagate down only when constraints change") needs no
clause here because it is already exact in the cache key: a child re-measures
precisely when the constraints handed to it differ, since a different `c` is a
different key and therefore a miss.

# It is a superset, deliberately

Every branch that cannot prove a bound answers [`None`] (drop everything), so
the failure mode is a redundant re-measure, never a stale box: a dirty node the
walk cannot reach from the root, an absent root, and a dirty root all widen to
the full pass.

# What `arrange(N, …)` additionally reads, and why it needs no clause here

[`LayoutEviction::arranges`] is the upward closure of `measures`, and that is
the *whole* extra analysis — because `arrange`'s other inputs are not style at
all. `arrange(N, origin, constraints, containing_block, …)` reads its own
**call arguments**, and those are not something a dirty set can bound: they are
computed by `N`'s parent from its parent's algorithm output. A later sibling of
a grown node is not dirty and never will be, yet it moves. Those inputs are
therefore checked by *value* at the point of the call, in the
[`ArrangeCache`](super::arrange_cache::ArrangeCache) key — not here. This function answers only "did anything in
`N`'s subtree change?"; the key answers "was `N` put in the same place, at the
same size, as last frame?". A subtree is reused only when **both** say yes.
