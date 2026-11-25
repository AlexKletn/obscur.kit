Routes `event` to the data drag ([`begin_drag`](UiRuntime::begin_drag)) that is
already in flight: a move tracks it across drop targets, a release drops or
cancels it, a cancel or a leave abandons it.

It returns a dispatch that reports the target and **nothing else** — no hover
change, no focus change, no command — because a drag consumes the pointer
rather than sharing it. That is the one behaviour a host would notice if this
merely ran alongside the click path: a release that ends a drag must not also
activate whatever it landed on.
