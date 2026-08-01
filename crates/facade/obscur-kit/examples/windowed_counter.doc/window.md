The live platform window, fetched from the backend via `native_window`
once the handler has materialized it (an [`Arc`] so wgpu can build a
`Surface<'static>` from it).
