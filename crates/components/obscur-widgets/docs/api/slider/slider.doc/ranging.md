A slider picking a RANGE between two values.

The ends are sorted here and nowhere else: a range whose low sat above its high
would paint backwards, and every reader downstream would have to sort it again —
one of them forgetting.

Sorted rather than clamped. Clamping the high up to the low would collapse the
range to a point and throw away one of the two numbers the author wrote, when
reversed ends almost always mean the pair.
