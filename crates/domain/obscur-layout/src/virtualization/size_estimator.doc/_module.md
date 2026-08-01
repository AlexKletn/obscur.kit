The [`SizeEstimator`] — estimated and measured item extents along the scroll
axis (spec §4.9 *estimated geometry*).

A virtualized list cannot afford to measure every item up front, so it starts
from a single `default_extent` estimate and refines it incrementally: as items
are really laid out, their measured extents are recorded and override the
estimate. The estimator maps between item indices and cumulative scroll offsets
by walking those extents.
