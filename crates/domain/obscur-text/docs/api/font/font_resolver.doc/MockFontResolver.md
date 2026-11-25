A deterministic reference [`FontResolver`](super::FontResolver) that maps a script directly to a
registered font (spec §5.10).

`MockFontResolver` ignores real glyph coverage entirely: it holds a table of
`ScriptTag -> (FontId, FontFace)` entries populated with
[`register`](Self::register), and [`resolve`](FontResolver::resolve) returns
the entry keyed by [`request.script()`](FontResolveRequest::script), or
`None` when the script is unregistered. This lets tests model a fallback
picture like "Latin uses font 0, Cyrillic uses font 1" without any font
tables; the coverage-based resolver arrives in a later step. Re-registering a
script replaces its previous entry (the last registration wins).
