Installs the port that parses **vector** sources (spec §5.17), replacing
any previous one and dropping the parsed-document cache.

Separate from [`set_resource_providers`](Self::set_resource_providers)
because a vector decoder is a genuinely different capability: a raster
decoder yields pixels, this yields geometry with `currentColor` still
deferred. A host may install either, both, or neither; with none installed
an SVG `src` resolves to nothing and paints nothing, exactly as an
unloadable bitmap does — never a crash, and never a rasterized fallback
that would silently break §5.17's vector guarantee.

A [`ResourceLoader`] is still required to fetch the bytes.

Replacement abandons outstanding vector loads and requests a frame to resolve
active SVG URLs.
