Creates a driver over the input domain `[min, max]`, starting with its
`value` at `min` (progress `0`).

The endpoints may be inverted (`min > max`); see the type docs. If they
are equal the domain is degenerate and progress is always `0`.
