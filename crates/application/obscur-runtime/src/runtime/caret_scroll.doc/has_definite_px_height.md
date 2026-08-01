Whether `style` carries a **definite** absolute-`px` `height` — a concrete
length, not `auto`/content-based sizing (the vertical analogue of
`has_definite_px_width`, which lives in `compose_only_edit.rs` with the
box-stability question it answers).

A definite-height field has a fixed box its content overflows (rather than
growing to fit), so it is the field that scrolls VERTICALLY with the caret. An
auto/percentage height returns `false`, so an auto-growing multiline field is
never treated as a vertical scroll container.
