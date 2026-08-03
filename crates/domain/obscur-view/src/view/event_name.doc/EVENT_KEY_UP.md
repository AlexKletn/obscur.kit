A key was released on the focused node — `on:keyup="…"`.

The runtime performs no default action on a release, so `prevent_default` has
nothing to suppress here. It is delivered for symmetry and because a release is
the only way an application can observe the end of a held key.
