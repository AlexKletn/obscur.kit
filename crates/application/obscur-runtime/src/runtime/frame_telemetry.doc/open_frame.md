Opens the frame span (spec §7.9) — the outer marker every phase span of
this frame sits inside.

Paired with [`close_frame`](Self::close_frame): a frame that opens the
span and records no phase tally is a frame an observer cannot tell from
one still running.
