Returns the asset density [`dpi_variants`] asks for at a device `scale`: the
top rung of the ladder (spec §5.16).

This is `ceil(scale)` capped at `@3x`, and `1` for any scale at or below `1.0`
as well as for a non-finite one. Two scales with the same density select the
same asset files, which is what lets a caller tell an *asset-relevant* scale
change (`1.0` → `2.0`: different files) from one that selects exactly the same
files (`2.5` → `3.0`: both `@3x`) and so needs no reload.
