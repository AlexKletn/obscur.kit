Registers (or replaces) the host-provided sfnt `font_bytes` for the run
font handle `id` (spec §5.2).

A [`DrawText`](obscur_scene::command::PaintCommand::DrawText) run tagged
with [`GlyphRun::font`](obscur_scene::text::GlyphRun::font)`() == id` is
then rasterized as real glyph
outlines against these bytes; runs whose handle has no registered font
fall back to box-coverage cells. Mirrors the Vello backend's
`register_font(id, bytes)` shape so a host registers the same id↔bytes into
both and the two backends draw the same faces. The bytes are stored (shared
via [`Arc`]) and a borrowed [`FontRef`] is rebuilt per draw.
