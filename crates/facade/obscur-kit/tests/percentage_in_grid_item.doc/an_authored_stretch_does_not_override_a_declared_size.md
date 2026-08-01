Writing `justify-self: stretch` / `align-self: stretch` by hand changes
nothing. The spec's condition is about the box's **computed size**, not about
how the alignment value was arrived at, so the author who asks for the stretch
out loud gets the same flex-start fallback as the one who inherits it from the
container's default.
