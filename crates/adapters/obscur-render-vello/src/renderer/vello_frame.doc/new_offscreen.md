Allocates an offscreen render target of `width` × `height` on `device`.

The extent is clamped to a minimum of `1` per axis (wgpu rejects a
zero-sized texture). The texture is created with the format and usages
Vello requires plus the copy/binding usages a host needs to consume the
result.
