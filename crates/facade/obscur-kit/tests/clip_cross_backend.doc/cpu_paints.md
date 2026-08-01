Whether the CPU backend's rasterized output is painted at `point` — the real
renderer, asked the only way a rasterizer can be asked. Probe points are pixel
centers, so the two backends are asked about the same spot without a cast.
