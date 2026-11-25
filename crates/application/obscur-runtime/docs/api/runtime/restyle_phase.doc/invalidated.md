Runs the match + style pass over **only** the nodes `dirty` records,
mutating the retained inputs and styles in place (§3.4 phase 5, §4.3).

The retained map is moved out and handed back as this frame's styles, so a
node outside `dirty` costs nothing at all — not a re-match, not a re-resolve,
not even a copy. What the phase pays is proportional to the dirty set, plus
the one pre-order walk `resolve_incremental` needs to visit the tree in
parent-first order.

Returns the styles, how many nodes were actually resolved, and the styles
they had before.
