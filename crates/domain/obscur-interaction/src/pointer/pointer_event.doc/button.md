The button whose transition this event reports — the DOM's `event.button`
standing beside its `buttons`.

[`buttons`](PointerEvent::buttons) is the set held *after* the event, and a set
cannot say which member moved: pressing the secondary button while the primary
is already down reports `{Primary, Secondary}`, and so does the opposite chord.
A context action and a click are two different presses, so the model has to be
able to tell them apart.

`None` means no button transitioned — a move, a crossing — or that the producer
does not say. A press or release naming nothing is read as the primary one,
because that is what every producer predating this field meant by it, and a
touch contact has no button to name.
