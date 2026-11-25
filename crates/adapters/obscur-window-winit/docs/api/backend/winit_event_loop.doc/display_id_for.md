Returns the [`DisplayId`] for a monitor named `name` appearing at
`ordinal` among its same-named peers, minting one on first sight.

Ids are retained forever, not cleared when a monitor disappears: a
monitor unplugged and plugged back in is the same monitor to the user,
and a caller holding its id across the gap should find it again rather
than meet a stranger.
