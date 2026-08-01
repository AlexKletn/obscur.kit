Attaches (or replaces) the primary host font (font handle `0`).

Back-compat shim over [`Self::register_font`]: single-font callers keep
working, and every untagged run (`font == 0`) renders against this face.
