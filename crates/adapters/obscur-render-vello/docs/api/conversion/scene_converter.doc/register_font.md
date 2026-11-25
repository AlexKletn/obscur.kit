Registers (or replaces) the host-provided [`FontData`] for the run font
handle `id` (`0` = primary; higher ids are fallback faces).

Fonts are a host resource; the adapter never fabricates one. The `id` must
match the face id the runtime's font resolver assigned (so scene run
handles line up with these fonts). Until a covering font is registered,
the run's [`PaintCommand::DrawText`] is skipped.

# Registering the same face twice is not a replacement

A `peniko::Blob` draws its id from a process-global counter, so the same font
bytes wrapped a second time are a **different** font as far as Vello is
concerned. That id is the key of Vello's glyph-outline cache
(`vello_encoding::GlyphCache::session` looks up by `font.data.id()`), which is
the one thing about a font that is genuinely expensive to rebuild — outlines
are extracted and encoded per glyph, and the cache is what stops the frame
paying for it twice.

So this stores a face only when it is not the face already held:

* **key** — the handle `id`; the value's own identity is
  `(FontData::data.id(), FontData::index)`.
* **invalidation** — the stored entry is replaced exactly when the incoming
  `index` differs or the incoming bytes differ from the stored blob's bytes.
  Nothing else can make it stale: a `Blob` is immutable, so the bytes behind a
  live blob id never change, and there is no time-, size- or frame-dependence
  to go wrong.
* **the test that catches a stale entry** —
  `registering_a_different_face_under_a_live_handle_replaces_it`: register face
  A under a handle, register face B (different bytes) under the same handle,
  and require that the face drawn under that handle is B. A guard that
  compared only handles, or only lengths, keeps A and fails it. Its pair,
  `registering_the_same_bytes_again_keeps_the_font_vellos_glyph_cache_is_keyed_by`,
  requires the blob id to survive — that is the whole point of the guard, and
  without it the guard could be "fixed" by deleting it.

The byte comparison is paid only when the handle is already occupied, which
outside the first frames means "never": the host registers each discovered face
once.
