The `ch` base for one node, measured only when that node actually needs it.

[`ComputedStyle::uses_ch`] is an allocation-free scan and answers `false` for
essentially every real node; a hit is what pays for the fallback chain and the
shaping call. So a stylesheet that never writes `ch` costs exactly what it did
before `ch` existed, which is the point — `ch` is a rare unit and must not tax
the nodes that do not use it.
