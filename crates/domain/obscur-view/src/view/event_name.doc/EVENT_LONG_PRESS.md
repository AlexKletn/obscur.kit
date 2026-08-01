Reserved symbol naming a **long press** — a direct contact held still past
the runtime's threshold without lifting (spec §6.16 `LongPress`).

The touch counterpart to a secondary click. A touchscreen has one button and
no modifiers, so everything a right-click or a modifier would reach on a
desktop is reached by holding still, which makes this the only route to
context actions on a tablet. Bound the same way a click is —
[`ViewEventBinding::new(EVENT_LONG_PRESS, command)`](super::ViewEventBinding::new)
— and the runtime fires `command` when the hold is recognized, *before* the
finger lifts, and instead of the click that lift would otherwise have been.
