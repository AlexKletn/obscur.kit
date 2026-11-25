Updates the load progress of `handle` (spec §5.16 progressive decode).

Sets the state to [`ResourceState::Loading`] carrying `progress`. This is a
no-op for an unknown handle.
