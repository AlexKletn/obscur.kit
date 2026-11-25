Returns whether `monitors` carry where each one sits in a global coordinate
space (spec §6.6: "On Wayland, *unavailable* global coordinates are
represented honestly").

The spec's condition is unavailability, not a platform, and the two are not
the same: a Wayland compositor publishes each output's logical position
through **xdg-output**, and winit reports it (`wayland/output.rs`'s
`position()` reads `logical_position`, falling back to `wl_output.geometry`'s
`location` — neither is hardcoded). Mutter, KWin and wlroots all implement it.
Withholding is a *may* in the protocol, not a rule, so keying off `is_wayland()`
blanks a topology the compositor published in full — which is the §6.6 rule
inverted on the one platform §6.6 names.

What is genuinely undetectable is a compositor that withholds by reporting
`(0, 0)` for everything: winit cannot distinguish that from a real origin. But
that ambiguity only exists when **every** monitor claims the origin, which no
real multi-monitor topology does — two monitors cannot both start at `(0, 0)`.
A single monitor at the origin is simply where it is. So the absence is
detected from the positions themselves and needs no `cfg`.
