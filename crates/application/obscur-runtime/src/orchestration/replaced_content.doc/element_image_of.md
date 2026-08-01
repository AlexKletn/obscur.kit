Reads a retained element node's own image `src` (an `<img>` / `Image`
primitive's source), if any — the key the runtime resolves against its image
cache.

Returns [`None`] for a node with no retained record or no `image_source`
attribute (every non-image element and every text node).
