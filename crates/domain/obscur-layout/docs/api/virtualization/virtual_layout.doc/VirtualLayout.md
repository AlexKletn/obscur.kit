Computes the materialized item range for a scroll position (spec §4.9).

A `VirtualLayout` pairs a [`SizeEstimator`] (the index ↔ offset mapping,
refined by measurement) with an [`OverscanPolicy`] (how far past the viewport
to materialize). This is the reusable virtualization mechanism; tables, trees
and grids build on it rather than reimplementing materialization.
