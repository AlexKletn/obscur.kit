Creates a line from its `runs`, source `range`, box `top`, box `height`,
and `baseline` offset.

The line [`width`](Self::width) is computed as the sum of the runs'
advances, and [`x_offset`](Self::x_offset) starts at `0` — set it with
[`with_x_offset`](Self::with_x_offset) once alignment is resolved.
