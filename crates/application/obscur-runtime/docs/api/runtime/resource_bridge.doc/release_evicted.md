Drops the hub bookkeeping for each URL the image cache `evicted` (spec
§5.19 eviction).

[`image_handles`](Self::image_handles) is the "already requested, do not
retry" guard the resolve pass consults. An evicted URL left in it would
never be requested again and its image would silently stop painting for the
life of the process — so eviction has to release the handle too, which is
exactly what lets a still-referenced image load again next frame.
