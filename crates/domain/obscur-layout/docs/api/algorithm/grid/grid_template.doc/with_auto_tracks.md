Returns a copy with the implicit-track sizing lists (`grid-auto-columns` /
`grid-auto-rows`) replaced.

Each list sizes the *implicit* tracks on its axis, cycled by the n-th
implicit track using index `n % len`. An empty list keeps the default
content-sized `auto` for that axis (byte-identical to the historical
all-`Auto` implicit tracks).
