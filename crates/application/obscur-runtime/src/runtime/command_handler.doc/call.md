Runs the handler against the propagating `event`.

The event is passed by mutable reference so anything the handler sets —
[`UiEvent::stop_propagation`], [`UiEvent::prevent_default`] — is visible to
the walk that called it, which is what makes those methods do something.
