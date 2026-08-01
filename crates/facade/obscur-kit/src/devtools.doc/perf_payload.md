Renders host-side frame performance ([`FrameStats`]) as the
[`Inspectable::Rendering`] snapshot payload (spec §7.9), so an external
DevTools client sees live FPS + CPU/GPU timings instead of a placeholder.

A host attaches it to the source it serves, e.g.
`inspect_source_from(&rt).with_snapshot(Inspectable::Rendering, perf_payload(&stats))`.
