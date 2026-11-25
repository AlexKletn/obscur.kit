Estimated and measured item extents along the scroll axis (spec §4.9
*estimated geometry*).

Every item is assumed to have `default_extent` logical pixels until it is
really measured; a measurement recorded via [`set_measured`](Self::set_measured)
then refines the estimate for that index. Cumulative queries
([`total_extent`](Self::total_extent), [`offset_of`](Self::offset_of),
[`index_at_offset`](Self::index_at_offset)) walk the per-item extents so the
mapping stays consistent as measurements accrue.
