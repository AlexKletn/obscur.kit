A trusted touchscreen contact: contact `contact` in `kind` at `(x, y)`.

Trusted for the same reason [`mouse`](crate::fixture::mouse) is — it models a
real finger, and the gated default actions (§2.3 step 4) only run for one.
Distinct `contact` numbers are distinct fingers, which is what lets a test hold
two controls at once.
