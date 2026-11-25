Everything the style phase reads **apart from** the retained tree and the
per-node interaction state (spec §3.4 phase 5, §4.3).

The match + style phase's inputs are exactly: the tree, the authored sheets,
the media environment (which here is a function of the viewport), and the
[`PseudoStates`] map. An incremental restyle is the claim that the *only* one
of those that moved is the interaction state — the tree is covered by the
frame's `reconciled` result, and this covers the rest. Any mismatch means the
frame cannot know which nodes a full pass would have changed, and must run one.

The sheets are held **by value** and compared structurally: a host is free to
hand `run_frame` a freshly built `Vec<Stylesheet>` each frame, so identity
would report a change that isn't one, and a stale-sheet restyle is silently
wrong rather than merely slow.
