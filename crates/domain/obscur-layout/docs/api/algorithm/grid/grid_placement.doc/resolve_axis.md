Resolves one placement axis to a zero-based track start and a positive span,
and indicates whether the grid limit changed that placement.
The index includes the prefix reserved across the grid for tracks before
the explicit start. A negative numbered line counts from the explicit end.
Placement can extend past either explicit boundary. Reversed edges are
ordered by position, and equal edges use one track.

Explicit anchors and spans are constrained by the implicit track limit.
An item wholly beyond the limit uses the nearest edge track.
