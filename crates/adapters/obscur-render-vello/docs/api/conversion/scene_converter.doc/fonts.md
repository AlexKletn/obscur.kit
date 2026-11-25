Host-provided faces, keyed by the scene [`GlyphRun`] font handle (`0` = primary).
Empty until [`Self::register_font`] / [`Self::set_font`] is called; text is
skipped while a run's font and the id-`0` fallback are both absent.

This is a **registration table, not a cache** — the distinction is the whole of
what follows, and it was worth writing down because "cache without a ceiling"
was the charge against it.

## Key

A `u32` run font handle. It is not this crate's to mint: the kit's
`FontRegistry` hands out `FontId`s densely in registration order, and
`InstalledFonts` forwards `(handle, bytes)` pairs to the renderer so run handles
and table keys are the same numbers. A key is therefore an index into the host's
face list, never a hash of anything, and two different faces never collide.

## Invalidation

Only [`Self::register_font`] writes here, and it writes exactly once per face:
the host's own faces at bootstrap, and afterwards whatever
`InstalledFonts::take_discovered` hands over — which is a **draining** queue, so
each discovered face is offered once and never again.

Nothing removes an entry. Re-registering the same id with the same bytes and
face index is a no-op on purpose (`register_font` says why: a fresh `Blob`
carries a fresh identity, and that identity is the key of Vello's glyph-outline
cache, so replacing a face with itself silently discards every outline cached
for it). Re-registering with *different* bytes replaces the entry.

## Ceiling

Where the keys come from, so where the ceiling is:

- The host's own faces, plus the bundled last-resort face: a small constant. For
  the on-screen keyboard this repository exists for, that is 4.
- One face per named `font-family` a stylesheet asks for that the machine
  actually has (`FontRegistry::discover_family`). A family the machine does NOT
  have registers nothing — the fontconfig adapter answers by `fc-list` and not
  by `fc-match`, so an invented name returns `None` and costs one `Box<str>` in
  `asked_families`. So sheets cannot inflate this by naming things.
- One face per character no already-registered face covers
  (`FontRegistry::discover`). This is the one that grows on a process that lives
  for weeks, and it grows by script rather than by character: the first Japanese
  character pulls in one CJK face and the next ten thousand pull in nothing.

Every branch resolves to a face the machine has installed, so the table is
finite, and its ceiling is the machine's font catalogue. That is finite but it
is not small. Measured on this desktop:

```text
fc-list, distinct files                       1043
their total size                              879 MB
largest single face (mingliub.ttc)             36 MB
largest of the Noto CJK faces                  26 MB
```

So the honest statement is: **bounded by the host's installed fonts, which is
not a small number.**

## What that costs here, and why the ceiling is not enforced HERE

An entry is a `u32` and a `FontData` — a fat `Arc` pointer, a `u64` blob id and
a face index, about 40 bytes — plus one pointer-sized allocation for the
`SharedFace` newtype. It is **not** the face. As of the pass that wrote this,
`VelloRenderer::register_font` shares the caller's `Arc<[u8]>` instead of
copying it (`vello_renderer.doc/SharedFace.md`), and the test
`a_registered_face_is_the_hosts_allocation_not_a_copy_of_it` holds that by
address. Before it, every face was resident twice — once in `FontRegistry`,
which never drops one, and once here — and the worst case above was 1.76 GB
rather than 879 MB.

Which is why a ceiling with eviction is not in this file. `FontRegistry` keeps
every face alive regardless of what this table does, so evicting an entry here
frees the 40 bytes and none of the megabytes: it would be a ceiling that
measures well and saves nothing. The bytes have one owner and the ceiling
belongs with the owner.

And eviction here is not merely useless, it is wrong: `font_for` falls back to
handle `0`, so a dropped face makes its runs draw the PRIMARY face's glyphs at
the dropped face's glyph ids — other letters, not missing ones — and the
draining delivery queue means it never comes back.
`dropping_a_face_would_draw_the_wrong_one_not_nothing` is that, as a test.

## Tests

| what | where |
|------|-------|
| a face is the host's allocation, not a copy | `vello_renderer.rs`, `a_registered_face_is_the_hosts_allocation_not_a_copy_of_it` |
| re-registering the same face is a no-op | `vello_renderer.rs`, `re_registering_the_very_same_allocation_is_a_no_op` |
| the same bytes twice keep the entry | `registering_the_same_bytes_again_keeps_the_font_vellos_glyph_cache_is_keyed_by` |
| different bytes replace it | `registering_a_different_face_under_a_live_handle_replaces_it` |
| the same bytes at another face index replace it | `the_same_bytes_at_a_different_face_index_are_a_different_face` |
| what eviction would do | `dropping_a_face_would_draw_the_wrong_one_not_nothing` |

There is no test for "the table stayed under a ceiling", because there is no
ceiling to test. The number a future pass would have to bound is
`FontRegistry::len`, in `obscur-text-harfrust`, and bounding it needs a
re-supply path first.
