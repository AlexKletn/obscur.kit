The row under `screen` **only if** its list registered a long-press handler
(spec §6.16).

A hold is armed only where the gesture is actually bound, so this asks the
same question as [`index_at`](Self::index_at) and the handler map together:
on a tablet the rows of a file listing are the main thing a user holds, and
they are precisely the nodes that cannot carry a binding of their own.
