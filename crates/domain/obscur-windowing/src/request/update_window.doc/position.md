A change to the position: outer `None` leaves it unchanged; `Some(Some)`
sets an explicit position; `Some(None)` clears it to platform-chosen.

The nested option is intentional: all three cases are meaningful, so it
cannot collapse to a single [`Option`].
