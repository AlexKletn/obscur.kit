Dispatches the `EVENT_LONG_PRESS` command bound at or above `node`.
Returns whether a handler ran.

The envelope reports [`EventOrigin::Gesture`], not `Pointer`: a long press
is what a **recognizer** concluded from a contact that stayed put, and §2.4
treats recognized gestures as their own source ("Competing recognizers use a
gesture arena"). It carries no pointer payload for the same reason — the
event that fires this one is the *passage of time*, not a pointer event, so
there is no position to report that the contact is still at. `trusted`
follows the contact that armed the hold: a recognizer cannot vouch for a
gesture the platform never reported.
