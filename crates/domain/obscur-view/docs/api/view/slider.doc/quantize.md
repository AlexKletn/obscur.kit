The nearest value this scale actually offers.

Snapped from `min` and not from zero, so a scale of 1..10 by 2 offers 1, 3, 5 —
the values the author declared.

When the span is not a whole multiple of the step, **the maximum is
unreachable**: 0..10 by 4 offers 0, 4 and 8, and asking for 10 answers 8.
Clamping to `max` instead would hand back a value the grid does not contain,
which is worse than a maximum nobody can reach. `<input type=range>` does the
same.
