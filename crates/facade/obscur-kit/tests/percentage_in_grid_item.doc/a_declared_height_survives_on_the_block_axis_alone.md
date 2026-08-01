The rule is per axis ("as appropriate to the axis"), so a declared `height` is
kept the same way a declared `width` is, and the *other* axis keeps stretching.

The row here is content-sized, so the block extent an unconstrained item would
take is its own content height; `height: 40px` must therefore read 40 while the
inline axis still fills the 211 track.
