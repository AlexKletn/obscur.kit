The **declarative** windowed counter — the `windowed_counter.rs` capstone
rewritten on the one-call [`obscur_kit::App`] entry point (spec §1.4).

`windowed_counter.rs` spells out ~890 lines to open one window: a winit
`ApplicationHandler`, the wgpu `Instance → Surface → Adapter → Device → Queue`
chain, the multi-font fallback stack + renderer font hand-off, the CSS compile,
swapchain resize/lost recovery, the present quad, and the pointer/key/IME arms.
[`App`] composes all of that from the kit's own facilities, so the app code
below is just the counter's state, view builder, CSS, and command handlers —
the *same behaviour*, an order of magnitude less boilerplate.

Run it on a real machine (a display + a GPU adapter are required):
`cargo run -p obscur-kit --example hello_app --features full`

Clicking the green bar increments the on-screen count and the red bar
decrements it; hovering a bar lights its `:hover` style. It cannot run in the
headless dev/CI environment (no display, no GPU), so there it is only compiled.
