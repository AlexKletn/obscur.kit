The plan must never narrow to nothing, and must always end at everything.

This is the guard on the defect the module doc describes: while the plan was
computed as `Backends::default().with_env()`, one exported `WGPU_BACKEND` — an
unavailable backend, or a misspelt one — made every attempt in it the empty set,
and the program exited before its first frame with no adapter it had ever
actually asked for.

It holds three properties of `attempts()` itself, not of a copy of it: no
attempt is empty, no attempt repeats an earlier one, and the last is
`Backends::all()`. Re-deriving the plan from the environment fails at least one
of the three for every value that has ever caused the fault — `gl` empties the
first attempt, `vk` makes the second a repeat of the first, and both move the
last away from `all()`.

Cheap and device-free: no instance is created, so it runs on a machine with no
GPU at all.
