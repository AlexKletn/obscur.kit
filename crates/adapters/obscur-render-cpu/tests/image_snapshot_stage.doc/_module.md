The §7.4 rendered-image stage, wired through the software backend.

`obscur-headless` deliberately holds no rasterizer, so [`ImageSnapshot`]
never *produces* pixels — it captures whatever raster a backend already made
(`ImageSnapshot::from_rgba`). This integration test is where the one
backend-dependent §7.4 stage is actually produced: the software
[`CpuRenderer`] rasterizes a scene into a `tiny_skia::Pixmap`, and its bytes
become the image snapshot. It proves the stage is deterministic (same scene
→ same image) and attaches into a [`PipelineSnapshot`].

The assertions here are **exact**, and deliberately so: one backend
rasterizing one scene twice must produce identical bytes, and a tolerance
would only weaken that. §7.14's tolerance is for *cross*-backend and
cross-font-version comparison, and lives in `golden_tolerance.rs`.
