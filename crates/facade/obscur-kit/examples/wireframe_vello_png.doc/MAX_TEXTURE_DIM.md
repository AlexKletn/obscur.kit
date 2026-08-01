The largest offscreen texture dimension this harness will ask the GPU for. The
hard `wgpu` limit is 8192, but the tall wireframe sheet (4160px logical × 2× =
8320) loses the device near that ceiling on some drivers, so we keep a safe
working margin: `VIEWPORT × scale` is capped to this on the longest axis.
