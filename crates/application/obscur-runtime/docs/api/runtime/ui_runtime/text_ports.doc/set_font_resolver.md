Installs a font `resolver` on the owned scene coordinator, replacing the
default empty [`MockFontResolver`](obscur_text::font::MockFontResolver).

Font resolution is a hexagonal port: the runtime only knows the
`Box<dyn FontResolver>`. Installing a coverage registry lets each itemized
script run resolve to a covering font, so the fallback orchestrator tags
runs with the right face indices for the renderer to pick per run. Leaving
it unset falls every run back to the primary face (font `0`).
