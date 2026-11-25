§11.5 sub-step (a) — grow the *base* of the tracks spanned by one item whose
**min** sizing function is `kind`, to fit that kind's contribution, then
record the per-track increase as this sub-step's planned increase (flushed by
[`flush_base`] so same-batch items don't double-count).

Tracks of another kind in the same span are left alone: they have their own
sub-step, with their own contribution. They do count toward the `sum` the
contribution is measured against, per §11.5.1 — the space to place is what the
item needs beyond *every* track it spans.
