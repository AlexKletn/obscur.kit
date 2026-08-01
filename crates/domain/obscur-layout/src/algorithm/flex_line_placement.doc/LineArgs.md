Inputs for [`place_line`]: one flex line's items plus the shared container /
style context. Grouped into a struct because the single-line sizing pass
legitimately needs this many parameters and a positional argument list would
be error-prone.
