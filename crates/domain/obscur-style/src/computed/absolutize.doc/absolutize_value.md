Returns `value` with every [`Length`] it carries absolutized against `ctx`.

The match is exhaustive rather than wildcarded so a new length-bearing
[`PropertyValue`] variant cannot quietly opt out of the pass and reintroduce
the layout/paint disagreement this exists to close.
