The half of the proof that needs the windowed feature set.

`App` owns a window and an event loop, so it is compiled only under
`window-winit` + `render-vello` and is absent from the default build these
tests otherwise run in. The test below is therefore SKIPPED by a plain
`cargo test`, which is why the mechanism it exercises is also asserted above
without it.
