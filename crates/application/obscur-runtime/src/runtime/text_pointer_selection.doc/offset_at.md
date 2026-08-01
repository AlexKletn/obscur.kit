Resolves the byte offset in the focused editable `field`'s text at pointer
`point` (spec §5.11): finds `field`'s text leaf, maps `point.x` into that
run's local x (`point.x − text-origin.x`), and asks the scene coordinator
for the nearest cluster boundary — the inverse of the caret-render advance
walk, so a click at the rendered caret x of offset `i` returns `i`.

The live **session** text is the source of truth for the field's glyphs
(matching what compose paints), and the retained box origin + computed style
(from the last produced frame) supply the run's position and shaping inputs.
Returns `None` when no active session covers `field`, the field has no text
descendant, or no box was retained for it.
