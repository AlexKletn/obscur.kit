The pointer half of §2.3, reached the way an application reaches it.

`keyboard_events.rs` closed the same fault on the key path: input was resolved
straight to a command, so there was no raw *event* — only the meaning the
runtime had already decided the input had. On the pointer path an application
could observe a `click` and a `longpress` and nothing else. It could not see a
press, a release, a move or a cancel; `prevent_default` had nothing to refuse,
because by the time anything was dispatched the focus had moved, the caret was
seated and the activation had run.

`pointerdown`, `pointerup`, `pointermove` and `pointercancel` are reserved names
now, dispatched to the hit target through capture / target / bubble like any
other event, mirrored in `name_binding`, and — the point of the ordering —
dispatched *before* the runtime's own behaviour for that kind, so each refusal
is reachable:

| refusing | suppresses |
|---|---|
| `pointerdown` | the focus move, the caret, the press anchor, gesture arming — and so any click that would have followed |
| `pointerup` | the click, its activation, the virtualized row click |
| `pointermove` | the live drag-selection |
| `pointercancel` | **nothing**, deliberately — see `pointercancel_is_reported_and_its_refusal_is_inert` |
| `pointerenter` / `pointerleave` | **nothing**, deliberately — see `prevent_default_on_a_crossing_is_inert` |

`pointerenter` and `pointerleave` are the second pair, and they are not about
the hit target at all: they name the nodes the *hover chain* crossed, which is
how one move can leave a row and enter its sibling while targeting only the
sibling. Leaves come first and innermost-first, enters second and
outermost-first, so the two sequences nest — that order is the claim, so the
cases assert a sequence rather than a count. The binding is resolved on the
crossed node alone, which is the one place these differ from every other event
here; the command it names still routes.

Two things a refusal deliberately does not reach, and both are asserted rather
than assumed: a pan already scrolling (the gesture owns the pointer through
capture before the event is delivered), and the bookkeeping a release owes —
ending the press, the capture and the pan. Declining those is not declining an
action; it is declining a fact.

These cases are written from the consumer's side, only `obscur_kit`'s public
API, because every one of them was a delivery gap and a delivery gap is only
closed at the boundary an application stands on.

Not a `spec_*` file: the spec-derived suites are an oracle written without
reading this repository. The §2.3 and §2.4 conformance statements live in
`spec_ch2_interaction.rs` and are not touched here.
