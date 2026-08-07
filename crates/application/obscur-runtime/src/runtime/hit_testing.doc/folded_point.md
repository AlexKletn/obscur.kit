`screen` moved into the space `node`'s layout box lives in: the point with
every scroll container ABOVE `node` scrolled back in.

The spatial index and [`LayoutBoxes`] are keyed by unscrolled layout boxes,
while a scroll container's descendants render translated by `−scroll_top`.
The fold puts those back — but only the ones `node` actually rides. Folding
once, globally, and matching the result against every candidate is the
defect this replaces: a `position: fixed` toolbar or an absolutely
positioned overlay that merely OVERLAPS a scrollport does not scroll with
it, so testing it at a point shifted by a scroll it never took made it
unclickable exactly when the region behind it was scrolled.

`scrolls` maps each scroll container to its clamped `scroll_top`; a node
with no scrolling ancestor folds to `screen` itself, so a document that
never scrolls lands byte-identically to one built before scroll containers
existed.

Note that this follows the LOGICAL ancestor chain, which is what compose
scrolls too: a `position: fixed` box nested inside a scroll container is
painted scrolled by this engine, and is therefore hit scrolled, both wrong
in the same direction rather than each wrong in its own.
