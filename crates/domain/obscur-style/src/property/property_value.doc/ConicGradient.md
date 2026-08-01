A `conic-gradient(...)` background image (see [`ConicGradient`]): the value
of `background-image` when it is an authored conic gradient. The gradient
carries its starting angle (CSS convention), box-relative center, and
resolved color stops; the scene coordinator turns those plus the painted
box's bounds into the concrete center and sweep start angle when it builds
the fill brush (spec §5.2).
