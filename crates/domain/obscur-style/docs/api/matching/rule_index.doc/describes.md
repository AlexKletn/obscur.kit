Whether this index was built from a sheet set of exactly this shape — same
number of sheets, same number of rules in each.

The index addresses rules by `(sheet, rule)` position, so handing it a
different set would read the wrong rules and cascade the wrong values,
silently. Callers check this and fall back to the unindexed walk when it
answers `false`; it is a guard against a stale index, not a substitute for not
keeping one.
