Returns whether the guarded work is now starved and should be promoted.

This is `waited >= threshold`, except that a `threshold` of `0` disables
the guard and always returns `false`.
