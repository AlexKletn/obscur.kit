Whether an image has been asked for and not yet arrived.

Kept on the loop side so [`request`](Self::request) is idempotent within a turn:
asking three times while one is in flight must not queue three acquires.
