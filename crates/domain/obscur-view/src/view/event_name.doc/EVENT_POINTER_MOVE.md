A pointer moved over the hit node — `on:pointermove="…"`.

Dispatched **before** the live drag-selection reads the move, so a listener that
calls `prevent_default` suppresses the text range that sweeping over a field
would otherwise select.

It is delivered **after** the pan recognizer has seen the move, and refusing it
does not stop a scroll already under way. A pan owns the pointer through
capture: by the time the event is delivered the gesture has already decided the
contact is dragging the container and not pressing what is under it, and the
target this event carries is the capture target for that reason. Refusing a move
mid-scroll would leave the container half-scrolled and the finger still down,
which is worse than either outcome the author was choosing between.

**Cost.** This fires at the pointer's report rate. Nothing is walked, allocated
or dispatched unless some node in the committed tree actually binds this name;
the runtime keeps that answer as a bit recomputed once per structural frame.
