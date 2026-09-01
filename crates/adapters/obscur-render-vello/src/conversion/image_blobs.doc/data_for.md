The `ImageData` for this image, reusing the one from last frame when the pixels
are the same allocation.

Returning a `Blob` with the id vello already has is the whole point: it turns an
atlas miss and a full re-upload into a lookup.
