A `radial-gradient(...)` background image (see [`RadialGradient`]): the
value of `background-image` when it is an authored radial gradient. The
gradient carries its box-relative shape / size / center and resolved color
stops; the scene coordinator turns those plus the painted box's bounds into
the concrete center and radius when it builds the fill brush (spec §5.2).
