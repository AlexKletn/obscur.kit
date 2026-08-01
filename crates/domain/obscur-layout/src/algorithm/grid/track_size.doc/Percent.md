A fraction of the container's content extent along this axis, stored as a
`0..=1` fraction (`30%` → `0.30`). Resolved to `fraction * axis_size`
inside the algorithm — where the axis extent is known — and, like
[`Fixed`](TrackSize::Fixed), consumes its space before `fr` distribution.
