Closes the step [`advance_flings`](Self::advance_flings) opened: stops the
coasts in `stalled` — the ones whose scroll did not move — and retires
every fling that has finished.

A coast that reaches the end of its content is **stopped** rather than left
decaying against the clamp: the list has arrived, and a fling that went on
being advanced would keep asking for frames while moving nothing.
