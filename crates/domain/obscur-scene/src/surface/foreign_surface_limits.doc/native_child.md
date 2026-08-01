Returns limits typical of a native child surface/window.

A native child surface is positioned and clipped by the platform, so
geometric operations remain available (`allows_transform` and
`allows_clip` are `true`), but per-surface compositing effects that the
platform cannot honour are restricted (`allows_opacity` and
`allows_blur` are `false`).
