Reverses playback, mirroring the current position so a running reversal is
seamless (spec §4.17 "reversal / interruption").

Flips the direction and mirrors the position within the current iteration:
`elapsed → duration - elapsed` and `progress → 1 - progress`. The position
on screen does not jump; only the direction of travel inverts.
