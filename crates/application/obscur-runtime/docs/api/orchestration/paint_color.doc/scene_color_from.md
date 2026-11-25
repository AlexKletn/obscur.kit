Converts an authored style color into a scene paint color.

[`StyleColor`] is the authored style color (straight, non-premultiplied sRGB
components); the scene [`Color`] is the paint color a renderer consumes.
Phase 1 copies the components verbatim — the linear/premultiplied conversion
the GPU needs is a renderer concern applied later, not in the coordinator.
