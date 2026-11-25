Composes a paint-ordered [`DisplayList`] of background fills from the
styled + laid-out tree.

Returns an empty (and balanced) display list when the tree has no root.
Otherwise the tree is walked pre-order from the root (parent before
children, children left-to-right, cycle-safe); for each visited node that
has a non-transparent computed `background-color` **and** an arranged box
in `boxes`, a fill of that box's rectangle is appended in paint order.

`computed` supplies each node's [`ComputedStyle`](obscur_style::computed::ComputedStyle)
(typically produced by the style coordinator) and `boxes` supplies each
node's absolute layout rectangle (produced by the layout coordinator). A
node missing from either map simply contributes no fill.

`editing` is the runtime's active text-editing session, or [`None`] when no
editable is focused. For the editing field's text (its
[`node`](EditingView::node) or that field's `Text` descendant) the compose
draws the session [`text`](EditingView::text) instead of the retained
content — so a just-typed character is visible immediately — and paints a
caret line (collapsed selection) on top of the glyphs plus a selection
highlight (non-empty selection) behind them. With `editing` = [`None`] every
text node paints its retained content with no caret or highlight, exactly as
before (the byte-identical regression path).
