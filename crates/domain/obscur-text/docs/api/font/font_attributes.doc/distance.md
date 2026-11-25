Returns a heuristic distance from `self` to `other`, where `0` means an
exact match and larger values mean a poorer fit (spec §5.18).

A provider uses this to pick the nearest available face for a request.
The distance is the weighted sum of three terms:

* the absolute weight difference (`|weight - other.weight|`, one unit per
  point on the `1..=1000` axis),
* a flat [`SLANT_PENALTY`](Self::SLANT_PENALTY) when the slants differ,
  and
* the absolute difference of the two [`FontStretch::percentage`] values
  (one unit per whole percent of width).

It is symmetric and monotonic in the weight difference, so a request for
a light weight ranks progressively heavier faces progressively worse.
