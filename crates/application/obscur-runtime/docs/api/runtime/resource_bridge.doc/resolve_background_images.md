Requests the URLs in freshly computed background layers through the resource hub.
Raster URLs populate the image cache; SVG URLs populate the vector cache.

Loading is deferred or dispatched to the host executor. A missing provider or
failed load leaves a failed handle and reports a diagnostic. The layer starts
painting once its resource has been decoded and committed.
