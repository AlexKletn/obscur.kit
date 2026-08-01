Reports the host's environment values — the safe-area insets and the
on-screen keyboard's geometry — which CSS `env()` resolves against (spec
§6.15). Returns whether anything changed.

This is the channel §6.15's "typed environment values" names, and the one
the kit did not have: a notch, a rounded corner, a gesture bar and a
keyboard are facts about the device and the moment, knowable only by the
layer that owns the window. An application cannot discover them and must
not guess them; it lays itself out around whatever arrives here.

**Dynamic by design.** These change while the app runs — the keyboard
comes and goes, the device rotates, a cutout moves from the top edge to
the side — so a change invalidates style and schedules a frame rather than
being read once at startup. Reporting the same values again is free and
schedules nothing.
