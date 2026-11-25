Records a `Down` for `pointer` on `target`, resolving its click count from
the pointer's own previous press.

Returns the target the pointer's **previous** outstanding press anchored,
which is what tells the caller whether the `:active` styling state changed.
The new press's click count is read back with
[`click_count`](Self::click_count).

A run cycles 1 → 2 → 3 → 1, so a fourth rapid click collapses back to a
single caret (matching platform multi-click behaviour) and the count stays
bounded. Two presses chain only when they share a target, fall inside
[`MULTI_CLICK_WINDOW_MICROS`] and land within [`MULTI_CLICK_RADIUS`].
