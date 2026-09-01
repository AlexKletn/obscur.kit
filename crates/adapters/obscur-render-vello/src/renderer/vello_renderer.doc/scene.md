The vello scene, kept between frames for its buffers.

Its contents are meaningless outside a render: every frame resets it before
encoding. What survives is the six allocations underneath, which a fresh
`Scene::new()` would give back to the allocator and immediately ask for again.
