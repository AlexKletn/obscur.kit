End-to-end: the in-window inspect overlay (spec §7.6) highlights the hovered
node's box and the runtime reports its readout, driven through the real
render + hit-test path (render → `handle_pointer` → hover → compose overlay).
