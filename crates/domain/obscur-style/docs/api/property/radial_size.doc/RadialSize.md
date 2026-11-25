The ending size (extent) of a CSS `radial-gradient(...)`: one of the four
extent keywords or an explicit radius length.

The extent keywords place the gradient's `100%` ring relative to the painted
box and the gradient's center; the scene coordinator resolves each against
the box at paint time. An explicit [`RadialSize::Length`] is a concrete
radius (an absolute `px` at paint time; a percentage is resolved against the
box). The CSS initial — used when no size is authored — is
[`RadialSize::FarthestCorner`].

An explicit [`RadialSize::Ellipse`] keeps the two axes independent.
Percentage radii resolve against the width and height, respectively.
