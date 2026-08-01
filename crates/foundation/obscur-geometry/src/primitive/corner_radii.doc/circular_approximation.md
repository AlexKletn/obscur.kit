The radius of the circle standing in for this quarter ellipse, for a backend
that cannot draw one — the mean of the two axes.

**This is a lossy fallback, not a CSS rule.** No single radius describes a
quarter ellipse, so any backend forced to pick one is approximating. The mean
is chosen because it is *exact* whenever the corner is already circular
(`x == y`, the case such a backend can express, so the common path never
degrades) and unbiased otherwise — unlike `max(x, y)`, which systematically
over-rounds, or `min`, which under-rounds.

Every caller must be a backend that genuinely cannot express an ellipse, and
must say so at the call site. Do **not** reach for this from scene geometry:
[`super::super::Rect`]-based path building expresses ellipses exactly, and
clip/scene geometry must agree across backends (spec §5.2).
