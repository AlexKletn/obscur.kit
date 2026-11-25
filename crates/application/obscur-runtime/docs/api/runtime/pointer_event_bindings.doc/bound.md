The set, as one bit per raw pointer event name.

A `u8` because seven names fit in one and the whole point of the gate is that
asking it costs nothing. `ALL_BITS` records what is left of it.
