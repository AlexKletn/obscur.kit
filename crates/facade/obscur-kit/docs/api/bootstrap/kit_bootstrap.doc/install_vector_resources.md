Injects the SVG vector decoder into `runtime` so an `<img src="...svg">`
paints as vector outlines that follow the computed `color` (spec §5.17).

Additive to [`install_image_resources`](Self::install_image_resources),
which supplies the loader this needs and handles bitmaps: a host may want
raster images without SVG, or both. Not calling it leaves the runtime's
honest-capability default — an SVG `src` paints nothing rather than being
rasterized behind the author's back, which would break §5.17's "SVG remains
vector through the scene pipeline".

The document is parsed once and cached with its `currentColor` unresolved,
so a theme change re-colours it without reparsing.
