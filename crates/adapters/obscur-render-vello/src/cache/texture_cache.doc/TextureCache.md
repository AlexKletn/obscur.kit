An in-memory map of [`wgpu::Texture`]s keyed by a content hash.

Uploading image data to the GPU is expensive; a scene that draws the same
image every frame should upload it once and reuse the resident texture. This
cache keeps each [`wgpu::Texture`] under a caller-computed `u64` content hash.
A `wgpu::Texture` is a cheap ref-counted handle, so caching and cloning it is
inexpensive.

Constructing the textures themselves requires a GPU device (owned by the
host, spec §2.7); this cache is only the storage, so its own construction and
emptiness are device-free while populating it needs a live device.
