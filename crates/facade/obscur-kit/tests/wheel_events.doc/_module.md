The wheel, and the scroll it can refuse — reached the way an application
reaches it.

The third pass of one fault. `keyboard_events.rs` closed it for the key and
`pointer_events.rs` for the pointer: input was resolved straight into the
meaning the runtime had already decided it had, so there was no raw *event*, and
`prevent_default` had nothing to refuse because the runtime had already acted.
On the wheel path an application could observe *nothing at all*. A notch went
into `handle_scroll` and a container came out moved. There was no way to consume
a wheel for a zoom, a horizontal carousel, or a canvas that scrolls its own
contents; `on:wheel` hashed like any other author name and nothing dispatched
it.

`wheel` is a reserved name now, dispatched to the node under the point through
capture / target / bubble like any other event, mirrored in `name_binding`, and
— the point of the ordering — dispatched *before* the container moves, so the
refusal is reachable.

# The payload

A notch is a delta and a position, and §2.4's pointer model is neither: it
describes a *contact*, with an id, buttons, pressure and a position it moved to.
So the wheel carries its own `EventPayload::Wheel`, read with `event.wheel()`.
Carrying it as a `PointerEvent` would have meant synthesising a `Move` that never
happened — a report the model could not then tell from a real one, which is what
`a_wheel_is_not_a_pointer_event` pins from both sides.

# A refusal is a claim

`handle_scroll` reports which node took the notch, and `App::on_scroll` is what
runs when nothing did — an application that windows its own list scrolls it
there. So a refused wheel reports the node that refused it, not `None`.
Reporting otherwise would scroll the host's list out from under a listener that
had just declined the scroll, which is the exact outcome the author was
refusing. `a_refused_wheel_is_a_claim_not_a_fall_through` asserts both halves:
the refusal names its node, and an unbound notch over nothing scrollable still
reports `None`.

# The node it names

The hit target, not the scroll container. A click at that point would land on the
row, so a notch there names the row — and the ancestor walk that resolves the
binding is what still lets a listener on the list hear it. Both directions are
asserted, because "targets the row" is only half a rule if a listener on the
container cannot hear it.

# Cost

`wheel` shares the raw pointer names' gate: nothing is hit-tested, walked or
dispatched unless some node in the committed tree binds the name.
`an_unbound_wheel_name_is_never_dispatched` is what that looks like from the
outside — a registered handler that stays quiet because nothing bound it.

These cases are written from the consumer's side, through `obscur_kit`'s public
API only, because this was a delivery gap and a delivery gap is only closed at
the boundary an application stands on.

Not a `spec_*` file: the spec-derived suites are an oracle written without
reading this repository, and nothing here touches them.
