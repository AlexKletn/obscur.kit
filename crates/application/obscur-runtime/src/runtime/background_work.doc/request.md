Enqueues one unit, reporting whether the queue **became** non-empty.

`true` on the `0 → 1` edge only: that is the single moment the scheduler
has to be told work is waiting, and re-announcing it on every later unit
would restart the fairness count that the waiting work depends on.
