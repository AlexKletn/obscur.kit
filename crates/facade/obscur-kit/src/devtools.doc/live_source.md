Builds the full live inspect source a windowed host publishes each frame:
[`inspect_source_from`] (tree/layout/styles/focus) plus the
[`Inspectable::Rendering`] frame-perf snapshot from `stats`. One call so a
host does not re-assemble the two every frame.
