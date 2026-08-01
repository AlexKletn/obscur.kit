The render scale: `CAP_SCALE` clamped to `1.0..=4.0`, **default 2.0**.

The Vello filter pass composites each filtered subtree in *device* space, and
the scene converter now maps the filter bounds + gaussian radius through the
root `device_scale` transform — so the `filter: blur()` card (§3) paints for
real at any HiDPI scale (it previously dropped out under `device_scale > 1`).
Supersampling at 2.0 renders everything crisply and also exercises that
device-scale filter path. `CAP_SCALE=1` renders 1:1.
