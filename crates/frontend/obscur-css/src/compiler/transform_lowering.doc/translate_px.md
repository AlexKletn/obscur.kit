Reads a `translate` distance in logical pixels: an absolute `px` (or a unitless
`0`). A **percentage** is treated as `0` (the box size is unknown at lower time
— a documented approximation); any other unit yields [`None`].
