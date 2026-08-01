Real text-shaping adapter over [HarfRust](https://docs.rs/harfrust), a
pure-Rust HarfBuzz port (spec §5.10).

This crate implements the domain's
[`TextShaper`](obscur_text::shaping::TextShaper) port with a real shaping
engine: it turns a [`ShapingRequest`](obscur_text::shaping::ShapingRequest)
into a [`GlyphRun`](obscur_text::shaping::GlyphRun) whose glyphs carry real,
font-specific glyph **indices** (not code points), advances and placement
offsets read from the font's `hmtx`/GPOS tables, and clusters that map back
to source byte offsets — with kerning, ligatures, and GSUB/GPOS applied.

The shaper needs font **bytes**: build one with
[`HarfrustShaper::from_bytes`] to wire a single host font, or
[`HarfrustShaper::without_font`] to rely on per-request
[`FontSource::Memory`](obscur_text::font::FontSource::Memory) bytes. Shaping
is infallible and deterministic: a missing font or unloadable bytes yield an
empty run rather than a panic.

Scaling: HarfRust reports advances/offsets in **font units**; this adapter
converts them to logical pixels with `font_size / units_per_em`, reading
`units_per_em` from the loaded face (never from possibly-placeholder
[`FontMetrics`](obscur_text::font::FontMetrics)). Bidi level resolution and
itemization happen upstream; this adapter shapes one already-uniform run.

**Line metrics are wired separately from shaping.** The port asks a shaper for
a font's vertical metrics by [`FontId`](obscur_text::font::FontId), and only
the [`FontRegistry`] that issued the ids knows the bytes behind each one, so a
[`HarfrustShaper::without_font`] shaper must be handed the registry's
[`FaceMetrics`] snapshot ([`FontRegistry::face_metrics`] →
[`HarfrustShaper::with_faces`]) or it will shape real glyphs into line boxes
sized by the idealized 0.8/0.2 ratios. Those fallbacks are counted by
[`HarfrustShaper::idealized_line_metrics`].
