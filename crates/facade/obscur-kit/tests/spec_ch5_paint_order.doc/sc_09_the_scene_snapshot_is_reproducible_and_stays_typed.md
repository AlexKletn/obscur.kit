§5.2 "Must match across backends: … - Scene/Paint snapshots;", contrasted with
"Allowed differences: - rasterization; - anti-aliasing; …". The snapshot is the
backend-independent artifact, so it must be reproducible with **zero**
tolerance and must not have lowered typed geometry into flattened paths.
