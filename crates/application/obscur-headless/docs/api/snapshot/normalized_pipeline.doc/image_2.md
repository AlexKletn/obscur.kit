Returns the rendered image, if a backend attached one (spec §7.4).

Borrowed rather than copied: an [`ImageSnapshot`] retains its raster for
the §7.14 tolerant comparison, so it is not [`Copy`].
