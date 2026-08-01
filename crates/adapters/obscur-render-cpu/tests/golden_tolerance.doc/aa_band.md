The indices of `img`'s partial-coverage pixels — the anti-aliased band.

Coverage is read off the alpha channel: the scene draws one opaque shape onto
a transparent surface, so alpha is 0 outside, 255 inside, and strictly
between exactly where the rasterizer anti-aliased an edge. This is the
operational definition of "where AA implementations are allowed to disagree".
