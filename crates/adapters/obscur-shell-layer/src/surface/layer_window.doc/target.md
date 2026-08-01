The raw handles a `wgpu::Surface` is built from.

# Panics

If the Wayland library hands back a null display or surface pointer, which
would mean the connection is already dead.
