Returns whether a pseudo-class is one the kit excludes (§4.13).

Three are excluded by policy — the dynamic-state `:link`, `:visited` and
`:target`. Two more are excluded because deciding them requires navigating
*forward* from the subject, which the matcher's `MatchElement` seam cannot
do: `:has()`, and `:nth-last-child()` **only in its `of S` form** (the plain
`:nth-last-child(An+B)` is decided from the sibling count and is fully
supported). All five lower to something that matches nothing, so the
diagnostic is what keeps the rule from being a silent drop.
