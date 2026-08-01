The retained domain record of a live window (spec §6.1, §6.2).

A `Window` bundles a window's stable identity with its desired
[`WindowProperties`], its size [`WindowConstraints`], its
[`WindowRelation`] to other windows, and its current display scale factor.
It is a pure data record; a backend owns the real platform window and
reconciles it against this model.
