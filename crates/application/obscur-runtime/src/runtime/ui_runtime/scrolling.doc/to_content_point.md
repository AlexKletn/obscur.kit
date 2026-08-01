Maps a host `position` (physical space) to content space via the inverse of
the root render transform `scale(s) · translate(−ox, −oy)` — the same
mapping [`handle_pointer`](Self::handle_pointer) applies before hit-testing.
