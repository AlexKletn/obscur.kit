Step 1c of the frame (spec §3.4): render every stale component and reconcile
its output into its own children, so a component-backed node's subtree exists
before style and layout read the tree.

It is a **borrow bundle**, not a holder: every field is a borrow of state the
runtime owns across frames, taken for the duration of one pass and released
with it. That is rule 2 of this crate's decomposition read literally — the tree,
the enactor, the telemetry and the diagnostics sink belong to the frame
pipeline, and a phase that kept any of them would be claiming to own the frame.

# Why the pass is one entity rather than four methods

Its interface is one call returning one `bool`; its internals are a bounded
fixpoint, a §1.7 error-boundary protocol with three distinct answers (retry,
fall back, propagate), a §2.12 slot projection and two performance counters.
The three answers are the reason it cannot be flattened: a *retry* commits
nothing and leaves the component stale so the frame end asks for another frame,
a *fallback* commits a different view over the same host, and a *propagation*
re-targets the commit at an **enclosing** component — so "what was rendered"
and "where it lands" are decided in different places, and the vocabulary that
connects them (boundary, escape, containment) is spoken nowhere else in the
runtime.
