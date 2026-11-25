Remembers the `ImageData` handed to vello for each image's pixels.

`Blob::new` stamps a globally unique id from an atomic counter, and vello's
atlas is keyed by exactly that id (`ImageCache::get_or_insert` looks up
`image.data.id()`). Building a fresh `ImageData` for every `DrawImage` command
therefore guaranteed a miss: every visible image was re-allocated in the atlas
and its whole `w x h x 4` bytes re-uploaded, every frame, while the previous
upload stayed resident until it aged out. The pixels never changed — the runtime
holds decoded images in its own cache and `shared_pixels` is an `Arc` clone — so
the upload bought nothing.

The cost is bandwidth, which is exactly what a mobile GPU has least of.
