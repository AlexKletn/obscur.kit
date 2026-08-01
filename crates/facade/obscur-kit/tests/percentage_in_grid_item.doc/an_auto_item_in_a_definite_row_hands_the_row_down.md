A control, matching [`an_auto_item_was_never_affected`] on the inline axis: an
`auto` item stretches to fill its row, and its child is half the row. **Green
before the fix too** — an `auto` item never routed through the item's own
percentage, so its used height reached its subtree unaided. Kept because it is
what shows the fix moved only what was broken.
