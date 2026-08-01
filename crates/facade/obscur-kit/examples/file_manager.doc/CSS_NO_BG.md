The same sheet with every `background-color` dropped — an overdraw probe.

The normal sheet stacks opaque fills: `.app` covers the window, `.list` covers
it again, and each of the ~76 rows paints its own. A path renderer that does no
occlusion culling pays for all of them. Running with `OBSCUR_NO_BG=1` renders the
identical tree and the identical text with none of those fills, so the difference
in `present` is the cost of the overdraw rather than of the geometry.
