Phases 2–3 of the frame (spec §3.4): match the authored sheets against the
tree and resolve the computed styles — either over everything, or over exactly
the nodes this frame can prove might differ.

A **borrow bundle** over the tier the phase drives (the matcher, the resolver
and the two maps they write) plus the three things it reads and never owns:
the tree, the live interaction state and the host environment. It is taken for
one style phase and released with it, so no phase can outlive the frame whose
tree it matched against.

# Why the scope and the two passes are one entity

Because the *only* thing that makes the incremental pass sound is the
question the scope asks. [`scope`](Self::scope) says "everything the match and
style phase reads, apart from the interaction state, is unchanged" — the
[`StyleKey`] comparison — and [`invalidated`](Self::invalidated) then treats
every node outside the dirty set as if it had been re-resolved and come out
the same. Split apart, a caller can run the incremental pass against a scope
computed from a different frame's inputs, and the result is a tree styled with
values nothing recomputed: correct-looking, silently stale, and invisible to
every test that renders a single frame.

The second pairing is the motion stash. [`invalidated`](Self::invalidated)
must undo the overlay *before* the retained map is treated as the resolver's
own output, and [`everything`](Self::everything) must clear the stash because
the map it described no longer exists. Both are writes to `overlaid` that only
make sense beside the pass that made them necessary.
