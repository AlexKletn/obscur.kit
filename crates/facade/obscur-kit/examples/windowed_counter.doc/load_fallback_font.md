Loads a *second*, different-script fallback font's bytes, for the multi-font
fallback demo.

The primary system font ([`load_system_font`]) typically covers Latin (and
often Cyrillic); this scans a few well-known CJK faces so a mixed-script label
has a covering font for the ideographs the primary lacks. Returns `None` when
none are present — the example still compiles and runs with only the primary
(the mixed-script glyphs then render as `.notdef`; see [`wire_text_stack`]).
