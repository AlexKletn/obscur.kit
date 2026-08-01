The half-open `[start, end)` range of item indices currently materialized
(spec §4.9 *materialization*).

`end` is never less than `start` — [`new`](Self::new) clamps an inverted range
to empty — so [`len`](Self::len) is always well-defined. Set operations
([`contains`](Self::contains), [`overlaps`](Self::overlaps),
[`union`](Self::union)) treat the range as the usual half-open interval.
