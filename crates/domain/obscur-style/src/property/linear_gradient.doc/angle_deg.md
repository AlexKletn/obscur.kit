The gradient line's angle in degrees (CSS convention: `0` = to top,
increasing clockwise). When [`corner`](Self::corner) is `Some`, this holds
the nominal diagonal angle (45°/135°/…) as a fallback, but the scene
coordinator computes the true aspect-ratio-aware angle from the box instead.
