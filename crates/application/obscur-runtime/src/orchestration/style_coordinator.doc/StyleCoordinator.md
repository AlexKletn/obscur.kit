Orchestrates computed-style resolution over a [`RetainedTree`].

The coordinator is **orchestration only** (spec §3.10): the computed-value
algorithm and inheritance live entirely in `obscur-style`'s [`StyleResolver`].
The coordinator's own responsibility is to walk the tree **parent-first**
(pre-order) and thread each node's already-computed parent style into the
resolver, so that when a node is resolved its parent's [`ComputedStyle`] is
already available for inheritance (spec §4.14).

The walk is cycle-safe and panic-free: a malformed or cyclic child link never
causes a loop, and a node that is unreachable from the root is simply not
resolved.

The same parent-first order is what makes `rem` resolvable at computed-value
time: the root **element** is resolved first (against the initial `font-size`,
which is what `rem` means *on* the root), and its absolute `font-size` becomes
the `rem` base every descendant is resolved against — see
[`ResolutionContext`].
