Renders every stale component and reconciles its output into its own children,
returning the restyle roots those sub-reconciles imply.

`Some(roots)` bounds the style phase to those subtrees; an empty vector means no
component's output differed from what was already retained, which is the steady
state. `None` means a sub-plan could not be bounded and the whole tree has to be
restyled — the same contract `RestylePhase::roots_of` uses.

Returning the roots rather than a bare "something changed" is what keeps an
interactive frame incremental. A component-backed key that changes one prop on
press re-renders one component; answering that with a whole-tree restyle made
every press cost the full O(nodes x properties) pass.
