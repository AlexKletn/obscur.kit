Connected, but the compositor does not implement `wlr-layer-shell`.

This is the expected outcome on GNOME (Mutter), which has declined the
protocol, and on any non-wlroots compositor that has not adopted it. It is
a capability answer, not a bug: a host should fall back to an ordinary
window rather than treat it as a failure.
