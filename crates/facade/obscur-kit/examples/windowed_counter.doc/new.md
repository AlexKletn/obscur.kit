Builds the application with a fresh [`UiRuntime`] and the two-button
counter UI, but no window, surface, or GPU device yet (those are created
on `resumed`). The runtime starts at a placeholder viewport matching the
initial window size; [`sync_viewport`](Self::sync_viewport) sets the real
physical extent once the surface exists.
