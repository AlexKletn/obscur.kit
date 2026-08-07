Whether a progress bar knows its own position, MIRRORED from
`obscur_widgets::progress::Progress::PROP_TRACKED` — see
[`PROGRESS_VALUE`](Self::PROGRESS_VALUE) for why a mirror and what it
costs.

It is load-bearing for accessibility and not only for painting: an
indeterminate bar still carries a value of zero, because it has to carry
something, and announcing that zero would say "not started" about a bar
that is running.
