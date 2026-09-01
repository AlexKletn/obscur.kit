Everything an application decides about a desktop surface, in platform-neutral
terms.

Hand it to [`App::layer`] and the ordinary application entry point opens a
panel instead of a window. Nothing here names a platform, a protocol or a wire
encoding; a backend translates the whole of it, and reports through
[`LayerFeature`](crate::port::LayerFeature) whichever parts it could not.

Two things a reader may expect and will not find:

- **No namespace or identifier.** The surface is identified by the
  application's own id, which the entry point already knows.
- **No sentinel sizes.** An axis the platform should decide is
  [`None`], not a zero.
