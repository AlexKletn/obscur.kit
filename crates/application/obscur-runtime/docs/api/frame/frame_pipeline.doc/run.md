Runs one synchronous frame over caller-owned state and returns its
[`FrameResult`].

The frame is short and **deterministic** (spec §3.4): for a given
`(tree state, desired, inputs, viewport)` it always produces the same
snapshot and the same flags. It never allocates cross-frame state of its
own and never panics.

The phases, in order:

1. **Reconcile** — the desired [`View`] is reconciled into `tree`,
   updating retained structure, identity, and text while preserving node
   ids across reorders. `reconciled` is set when the returned plan is
   non-empty (the tree changed).
2. **Style** — computed styles are resolved for every node reachable from
   the root, inheriting parent-first. `restyled` is set when any style was
   resolved.
3. **Layout** — the computed styles are mapped to layout styles and the
   tree is arranged under [`FrameContext::viewport`], producing absolute
   boxes. `relaid_out` is set when any box was produced.
4. **Compose** — a paint-ordered display list is composed from the styled,
   laid-out tree, installed as the scene root (advancing the scene
   generation), and published as an immutable snapshot. `composed` is
   always set: a scene is composed every frame, even if empty.

Input dispatch, motion sampling, and render submission are not part of
this core and are wired in later phases.
