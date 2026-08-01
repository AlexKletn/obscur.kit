Buffers a converted `event` for `id`, keeping [`BackendState::focused`] in
step with the compositor's focus verdict (spec §6.3).

Focus tracking lives here rather than inline in the handler so it is
reachable without an [`ActiveEventLoop`]: the handler's callback can only
run inside a real winit loop, but this — the part with a decision in it —
is what `activate` depends on.
