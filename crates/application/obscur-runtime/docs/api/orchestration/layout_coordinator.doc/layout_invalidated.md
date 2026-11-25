Lays out like [`layout_editing`](Self::layout_editing) but re-measures
**only** the layout contexts `dirty` can have changed, serving every other
node's size from `cache` (spec §3.4 step 8 "perform incremental layout",
§4.3).

This is layout's half of the §4.3 model, and it is the twin of
[`StyleCoordinator::resolve_incremental`](super::StyleCoordinator::resolve_incremental):
the same walk as the full pass, with the expensive step gated on a typed
dirty set. `dirty` = [`None`] means "this frame cannot say", which drops the
whole cache and reproduces [`layout_editing`](Self::layout_editing) exactly.

# What is incremental, and what is not

**Both halves are.** `measure` skips a node whose `(node, constraints)` is
memoized; `arrange` skips a whole **subtree** whose [`ArrangeKey`](super::arrange_cache::ArrangeKey) is
unchanged and whose entry survived eviction, serving its retained absolute
boxes instead of re-running its layout algorithms.

What is **not** skipped is the propagation itself, and that is deliberate:
§4.3's rules are *performed*, never inferred. A node whose subtree changed
re-runs its algorithm in full, which recomputes its children's origins and
sizes; a child then re-arranges precisely when that recomputed key differs.
So a sibling's growth still moves every later sibling and a flex line still
re-flows whole — those nodes miss on `origin`, not on dirtiness. The dirty
set alone would be a *silent* mispaint here; see [`ArrangeCache`].

Two O(nodes) costs remain by design: a served subtree is still walked to
copy its retained boxes out (a map insert per node, no algorithm and no
measure), and the returned map is still built whole. Layout algorithm runs
and text shaping are the costs removed, not the walk.

# What the caller owes

`cache` and `arrange` must hold only entries this same coordinator produced
over a tree, style set, shaper and hyphenator that differ from this frame's
in **no** way other than what `dirty` records, and `dirty` must record
**every** node whose layout inputs changed — see [`invalidated_layout`] for
the exact dependency list a retained entry rests on. A node missing from
`dirty` is not checked; its retained size and position are *assumed* still
correct. A caller that cannot promise that must pass [`None`], which is
always sound. The two caches are a matched pair: this method keeps them
consistent (one classification drives both), so a caller that clears one by
hand must clear the other.
