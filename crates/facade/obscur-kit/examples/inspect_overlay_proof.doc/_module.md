**In-window inspect overlay → PNG proof (spec §7.6).**

Renders a small row of cards, hovers the middle one through the real
hit-test path, turns the inspector on, and rasterizes the frame — so the
translucent-fill-plus-outline highlight the running window shows on hover is
captured as `target/inspect_overlay_proof.png` without a display.

Run it:
`cargo run -p obscur-kit --example inspect_overlay_proof --features render-cpu`
