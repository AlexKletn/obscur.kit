The protocol's signed encoding: `-1` ignore, `0` none, `n > 0` reserve.

Saturates at [`i32::MAX`] rather than wrapping, because a reservation wider
than the screen is nonsense either way and a wrapped negative would silently
become the *ignore* sentinel — the one value it must never turn into by
accident.
