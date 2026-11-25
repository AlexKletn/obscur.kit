Stylesheets shipped as part of the default theme package (spec §2.6).

§2.6 opens this crate's charter with the word **CSS**, and says what the crate
is: "это package ресурсов, а не домен" — a package of resources, not a domain.
A stylesheet is exactly that resource, so the UA default sheet's *text* lives
here, beside the tokens, presets and assets it shares a charter with.

What does **not** live here is the compilation. Parsing and compiling CSS is
`obscur-css`, a `frontend/` crate, and §2.11 runs that edge the other way —
`frontends → components`. So this module ships the source and the facade
compiles it; see `obscur_kit::default_stylesheet`.
