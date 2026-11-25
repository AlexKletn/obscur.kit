Creates an event contract for `name` carrying payload type
`payload_type`.

The event starts non-bubbling and non-cancelable; use
[`EventContract::bubbling`] and [`EventContract::cancelable`] to refine
it in a builder style.
