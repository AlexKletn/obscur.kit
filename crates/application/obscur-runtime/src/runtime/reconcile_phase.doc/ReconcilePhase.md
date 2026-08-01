Step 1 of the frame (spec §3.4): materialize the desired tree and settle the
retained tree against it — realize the virtual collections, commit the root
reconcile, mount and unmount the component instances, and render them — so
that style and layout read a tree whose structure is final.

It is a **borrow bundle**, not a holder: every field is a borrow of state the
runtime owns across frames, taken for the duration of one phase and released
with it. Nothing here survives the call, which is what keeps the runtime the
owner of the frame and this the thing that runs one step of it.

# Why the phase is one entity rather than a run of statements

Its interface is one call taking a [`DesiredView`] and returning "did the tree
change, and which subtrees can have to be restyled"; its internals are a
four-stage ordering whose every edge is a correctness constraint rather than a
preference:

- the desired tree is materialized **here** and not at the call site, because a
  reactive frame's builder reads state that this frame's input dispatch may
  have just committed to;
- §4.9's `<VirtualFor>` realization has to run **before** the reconcile and
  against this frame's viewport, since what reconciles is the realized window
  rather than the authored marker;
- the component sub-renders run **after** the root commit, because a
  component-backed node has to exist before an instance can mount on it; and
- the restyle roots this phase reports are only bounded while nothing widened
  them — a removal, or a component sub-reconcile that edited the tree through a
  plan this frame never sees.

That last point is the reason the return value is a pair rather than a `bool`:
"the tree changed" and "here is the part of it that changed" are two different
answers, and the second one degrades to [`None`] independently of the first.
