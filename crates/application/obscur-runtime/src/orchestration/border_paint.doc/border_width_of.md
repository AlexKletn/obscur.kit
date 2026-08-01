Reads a node's computed `border-*-width` (`id`) in logical pixels, defaulting
to `0.0` when absent or not an absolute pixel length.

Reading only `px` is correct rather than lossy: the computed value of a
`<length>` is already absolute, because the style resolver absolutizes every
length it produces (`em` / `rem` / `vw` / `vh` are folded there, once). This is
what makes paint agree with layout about the same box — a `.1em` border used to
reserve space here and paint nothing.

Two kinds of length still arrive un-absolutized, and both yield `0` — no
painted edge:

* a **percentage** (`border-width` takes none, so this is unreachable for this
  property; the radius corners take one and resolve it themselves — see
  [`corner_radius_of`]);
* a **`ch`**, which the resolver cannot fold without a shaper.
