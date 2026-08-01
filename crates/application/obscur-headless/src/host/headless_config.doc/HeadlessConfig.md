The configuration for a [`HeadlessHost`](super::HeadlessHost) (spec §7.4).

A headless run has no OS window and no GPU, so the only inputs it needs are
the ones that shape the frame pipeline's geometry: the logical `viewport`
each frame is laid out under, and the display `scale` (DPI) that maps logical
to physical pixels. Theme, locale, and deterministic-resource configuration
are defined by later phases and are intentionally absent here.

The type is a small plain value (`Copy`), so a host can hold it by value and
hand copies out freely. [`Default`] is an 800x600 viewport at
[`ScaleFactor::IDENTITY`].
