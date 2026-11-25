Registers (or replaces) a host-provided font under the run font handle
`id` (`0` = primary; higher ids are fallback faces).

Fonts are a host resource, mirroring the host-provides-device pattern:
the adapter never bundles one, so the host reads a font file and hands
the bytes here. They are wrapped in a peniko [`FontData`] (sfnt face
index 0) and forwarded to the scene converter, which draws each
[`DrawText`](obscur_scene::command::PaintCommand::DrawText) run against
the font whose id matches the run's
[`font`](obscur_scene::text::GlyphRun::font) handle.

The `id` **must** match the face id the runtime's font resolver (e.g.
`FontRegistry`) assigned to the same bytes, so scene run handles line up
with these fonts. Until a covering font is registered, its runs are
skipped.

**The bytes are shared, not copied.** The parameter takes anything that is
already an `Arc<[u8]>` — which is what the kit's font registry holds — and
keeps that allocation rather than duplicating it; see `SharedFace.md` for how,
and `scene_converter.doc/fonts.md` for what that means for how large this
table can get. A caller with a `Vec<u8>` still works and still pays one move
into an `Arc`, as before.
