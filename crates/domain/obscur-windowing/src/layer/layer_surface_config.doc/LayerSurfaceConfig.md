Everything a compositor needs to place a layer surface, as plain data.

This is the whole request. It carries no connection, no protocol object and
no platform type, which is what lets it be built, inspected and tested
without a Wayland session — and what lets a non-Wayland backend refuse it
honestly rather than approximate it.

# Size and anchoring interact

The requested [`size`](Self::size) is only honoured on an axis the surface is
**not** stretched along. A top bar anchored left+right gets the output's
width whatever it asked for; only its height comes from here. A surface must
therefore take its real extent from the configure event, never from this
request. See [`LayerAnchor::stretches_horizontally`].

A zero on a stretched axis is the idiomatic way to say "you decide".
