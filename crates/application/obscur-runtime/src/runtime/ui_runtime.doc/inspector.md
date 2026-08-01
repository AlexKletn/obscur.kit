The §7.6 DevTools projection of the live frame: the structured tree
snapshot, the hovered-node readout, the in-window highlight, and whether
that highlight is on.

One field rather than the bare flag it replaces, because the projection is
what the flag is *for* — see [`TreeInspector`], which owns the synthetic
part id-space the snapshot mints out of.
