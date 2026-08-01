**Agent-style headless inspection of the showcase scene (spec §7.6).**

Renders the shared showcase scene at a configurable viewport (env
`INSPECT_W` / `INSPECT_H`, default a tall tiled-window shape) and prints the
top of the tree via [`UiRuntime::inspect_snapshot`] — the same structured
observation an agent reads in a run, no window required.

Run it: `cargo run -p obscur-kit --example showcase_inspect --features render-cpu`
