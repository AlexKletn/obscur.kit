Clears focus when the focused node is no longer in the tree.

The defect this closes: a structural frame could remove the focused node and
nothing noticed. `focused` went on naming a dead id — harmless-looking, because
every dispatch that looked the node up simply found nothing and did nothing.
The accessibility projection is the place it stopped being harmless:
`SemanticTreeBuilder::build(tree, focused)` is handed the field verbatim, so the
platform was told focus sat on a node that had ceased to exist.

Run once per reconciled frame, beside the pointer-binding rescan and for the
same reason: a commit that carried an operation is the only moment the answer
can change.
