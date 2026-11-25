Ends `pointer`'s sequence: discards its hold, ends its pan, releases its
capture and drops the text drag — **the four together**, because they are
one interaction and a half-torn-down pointer is a visible bug.

Returns the pan that was still mid-drag when a [`Released`] pointer lifted,
which the caller may throw into a coast with
[`launch_fling`](Self::launch_fling). [`Cancelled`] never returns one: the
system took the sequence away, so nothing was thrown.

The outstanding *press* is not taken here — the `Up` path needs it before
the teardown, to resolve what the release activates, and the `Cancel` path
discards it without a click. See [`take_press`](Self::take_press).

[`Released`]: PointerCompletion::Released
[`Cancelled`]: PointerCompletion::Cancelled
