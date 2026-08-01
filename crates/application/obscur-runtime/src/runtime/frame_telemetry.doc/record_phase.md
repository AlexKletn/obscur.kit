Records one frame phase (spec §7.9): a [`PerformanceSpan`] marker for the
phase attributed to `frame`, a [`PerformanceCounter`] carrying the phase's
deterministic cost `metric`, and — when `metric` exceeds that phase's
bound — the resulting [`BudgetViolation`].

`metric` is taken **once**: the number the counter reports is by
construction the number the budget was checked against, and the bound is
the one belonging to `phase`.

A phase span knows its frame and nothing narrower — there is no single node
a whole layout phase is about — so `node`/`resource` stay empty rather than
being invented.
