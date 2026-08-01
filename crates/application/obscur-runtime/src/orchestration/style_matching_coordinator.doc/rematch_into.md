Re-matches **only** the nodes `dirty` records, updating `inputs` in place
(spec §3.4 phase 5, §4.3).

This is the incremental counterpart of
[`build_inputs_with_pseudo_env`](Self::build_inputs_with_pseudo_env): where
that runs every rule against every element, this runs them against the
invalidated elements and leaves every other node's retained entry alone.
The caller owes the precondition — `inputs` must be what the *same* sheets
and `environment` produced over the *same* tree, so that a node outside
`dirty` provably still matches what it matched then. That is the whole
contract: the accumulator says what changed, and only that is recomputed.

The re-match is a full re-match of each dirty node, not a patch: a node that
matched a rule last frame and matches none now has its entry
[cleared](StyleInputs::clear_cascaded), so a stale cascade cannot survive a
`:hover` ending.

Returns the number of elements re-matched. Non-element and stale nodes carry
no CSS identity, so they are skipped exactly as the full pass skips them.
