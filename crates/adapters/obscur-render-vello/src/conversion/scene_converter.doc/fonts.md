Host-provided fonts keyed by the scene [`GlyphRun`] font handle (`0` =
primary). Empty until [`Self::register_font`] / [`Self::set_font`] is
called; text is skipped while a run's font (and the id-`0` fallback) are
both absent.
