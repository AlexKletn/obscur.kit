Injects a filesystem image loader + the raster-image decoder into `runtime`
so its `background-image: url(...)` backgrounds resolve against files under
`sandbox_root` (spec §5.15 file loader, §5.16 decode, §2.7 adapters keep
host reach explicit).

The facade constructs the concrete adapters (a sandbox-rooted
[`FsResourceLoader`](crate::resource_fs::loader::FsResourceLoader) + the
[`ImageDecoder`](crate::resource_decode::decoder::ImageDecoder)) and hands
them to the runtime as domain ports, so the application layer depends only
on the ports — never on these adapter crates. A `url()` path resolves
relative to `sandbox_root` and cannot escape it. Not calling this leaves the
runtime's honest-capability default: `url()` backgrounds paint nothing.
