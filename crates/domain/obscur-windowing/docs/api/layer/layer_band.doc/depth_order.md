The band's position in the stack, `0` at the wallpaper and `3` at the top.

A comparable number rather than an [`Ord`] derive, so the ordering is stated
once in the body of a function instead of being implied by the declaration
order of the variants, where a later insertion would silently change it.
