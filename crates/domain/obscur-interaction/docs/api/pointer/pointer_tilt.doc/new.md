Creates a tilt, clamping both angles into `-90..=90` degrees.

Clamping rather than trusting the caller: a platform that reports tilt in
a different unit, or an uninitialised field, would otherwise hand an
application an angle no physical stylus can be at.
