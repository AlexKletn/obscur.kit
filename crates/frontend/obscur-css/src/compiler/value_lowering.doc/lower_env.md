Lowers `env(<name>)` / `env(<name>, <fallback>)` — a host-reported
environment value such as `safe-area-inset-top` (spec §6.15, css-env-1).

A **modelled** name lowers to [`Length::Env`], which resolves at
computed-value time against whatever the host has reported, so a keyboard
appearing or a device rotating changes the used value without reparsing the
sheet. Its fallback, if written, is accepted and unused: every modelled value
is always set — a screen with no cutout reports zero rather than nothing — so
there is no unset case for a fallback to cover.

An **unmodelled** name is a different matter. CSS says an `env()` naming an
unset variable falls back if a fallback is given and invalidates the
declaration otherwise, and that is exactly right here: `env(titlebar-area-x,
0px)` should lay out at `0px` rather than dropping the property, while a bare
`env(safe-area-inset-topp)` is a typo that must not silently behave like an
inset that happens to be absent.
