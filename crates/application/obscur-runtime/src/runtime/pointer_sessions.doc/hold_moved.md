Tells `pointer`'s armed hold, if any, how far the contact has travelled.

A contact that wanders is doing something other than waiting, so the hold
is ruled out before the pan is even consulted — the same travel that starts
a drag ends a hold, and a context action firing in the middle of a scroll is
the failure this prevents.
