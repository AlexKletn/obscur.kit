A cached wgpu pipeline set implementing a separable gaussian blur + composite.

Built once via [`GaussianBlur::new`] and reused across frames; each
[`GaussianBlur::render`] call only allocates the small ping-pong textures and
uniform buffers a single blur layer needs.
