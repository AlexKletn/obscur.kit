A progress bar's completed fraction on the `0..=1` scale the bar is drawn
against, read from the same
[`PROGRESS_VALUE`](Self::PROGRESS_VALUE) property the painter reads — so
what is announced is what is drawn.

A bar that is not tracked, or that states no value, is INDETERMINATE — it
is running and does not know how far along it is — and answers
[`SemanticValue::None`]. That absence is the accessible spelling of
"unknown": ARIA omits `aria-valuenow` for the same case. Reading the zero
an untracked bar happens to carry would announce "not started", which is a
different and false thing.
