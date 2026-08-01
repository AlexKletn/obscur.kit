Installs a real text `shaper` on the owned scene coordinator, replacing the
default deterministic [`MockTextShaper`](obscur_text::shaping::MockTextShaper).

Text shaping is a hexagonal port: the runtime only knows the
`Box<dyn TextShaper>` — it never depends on a concrete engine. A host (or
the facade) builds a real shaper (e.g. the HarfRust adapter, wired to the
same font bytes handed to the renderer so glyph indices align) and installs
it here. From the next composed frame on, each `DrawText` run carries that
shaper's glyphs (real, font-specific glyph **indices** rather than the mock's
Unicode-scalar placeholders). Leaving it unset preserves headless
determinism, so all default-path tests are unchanged.
