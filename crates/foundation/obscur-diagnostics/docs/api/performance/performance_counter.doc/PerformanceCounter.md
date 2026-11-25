A named monotonic counter attributed to a subsystem (spec §7.9).

Counters track cumulative event tallies for a subsystem — e.g. dispatch
count, reconciliation passes, CSS matches, text shaping runs, layout passes,
animation ticks or hit-tests. The value only ever grows via
[`increment`](Self::increment)/[`add`](Self::add) until an explicit
[`reset`](Self::reset) (typically at a frame boundary).
