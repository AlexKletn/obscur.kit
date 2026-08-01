The clock text a background thread keeps current, and the panel reads.

This is the point of the waker: nothing the compositor does makes a clock
tick, so without a way to reach the loop from outside it, the time would only
ever change when something else happened to wake it.
