**A desktop panel** — a `wlr-layer-shell` bar across the top of the screen
(spec §6.5).

This is the same toolkit as every other example: the same `View`, the same
CSS, the same runtime, the same touch handling. The only thing that differs
is how the surface reaches the screen — anchored to three edges of the output
instead of floating as an application window, and **reserving its own
height**, so a maximised window stops at the bar rather than sliding under it.

```sh
cargo run -p obscur-kit --features full --example panel          # compositor picks the output
cargo run -p obscur-kit --features full --example panel -- eDP-1 # pinned to a connector
```

Needs a compositor that implements `wlr-layer-shell` — Sway, Hyprland, river,
Wayfire, KDE. On GNOME the run ends with a stated capability error rather than
a crash, which is the point of returning one.

# What to try

Tap or click the buttons: they carry `:hover` and `:active`, and the tap
counter is ordinary application state re-rendered through the same reactive
loop a window uses. The bar is translucent, so whatever the compositor does
with the wallpaper behind it shows through.
