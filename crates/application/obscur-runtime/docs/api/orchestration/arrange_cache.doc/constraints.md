`min.width`, `min.height`, `max.width`, `max.height`.

The constraint's **percentage basis** is deliberately NOT part of this key,
where [`measure_key`](super::measure_cache::measure_key)'s twin does carry it. A node's basis is its parent's
content box (or, for a grid item, its grid area), and it moves whenever the
parent's own size moves — so keying on it would re-arrange every clean
child of every node that merely got taller, which is exactly the O(nodes)
walk this cache exists to avoid (`a_dirty_leaf_rearranges_its_ancestors_
but_not_the_siblings_above_it` measures it). The basis only changes an
arranged subtree through the node's own percentage size, and that size
lands in `constraints` — the node is arranged under a *tight* range — so in
practice a basis that matters has already moved the key. Where it has not,
the served subtree is stale; that gap is recorded rather than paid for.
