The focus half of §2.3, reached the way an application reaches it.

`keyboard_events.rs` and `pointer_events.rs` closed the same fault on the key
and pointer paths: input was resolved straight to a command, so there was no raw
*event*. Focus was the last of the three, and the plainest statement of the gap
is `docs/api/06-input.md`'s own "Not built" list, which said in as many words
that `focus` and `blur` "are not reserved names, because nothing emits them". An
application could see a `click`, and could not see the field it had just left.

`focus` and `blur` are reserved names now, dispatched from the one place focus
changes — so a path that moves focus later moves it through the same door and
the pair fires without that path having to remember to fire it.

# What these cases pin

- **The pair, in order.** `blur` on the node losing focus, then `focus` on the
  node gaining it, asserted as a sequence rather than as two counts.
- **Both ends of a transition.** Focus arriving from nothing fires `focus`
  alone; focus cleared to nothing fires `blur` alone. Neither end is silent.
- **A move that did not happen announces nothing** — re-pressing the focused
  node, and pressing a disabled control, which is deliberately not a blur.
- **Neither half is a gate.** `prevent_default` on either is inert, because
  focus has already moved when the event is dispatched. The refusal that exists
  is one event earlier, on `pointerdown`, and it is asserted here too — with
  its unrefused half beside it, so the silence is attributable to the refusal.
- **`change` before `blur`**, which is the order HTML fires the two in and the
  order a form reading a field on blur depends on.
- **The binding resolves through the ancestor chain**, which is a deliberate
  deviation from the DOM and is pinned by the case that forced it: a press on a
  field's glyphs focuses the field's *text leaf*, so an `<input on:focus>`
  resolved on the focused node alone would fire or not depending on where in
  the field the user clicked. Its consequence — a panel hearing a row inside it
  take focus, which is the DOM's `focusin` — is asserted too, along with the
  nearest binding winning so an ancestor never doubles a bound row.

These cases are written from the consumer's side, only `obscur_kit`'s public
API, because every one of them was a delivery gap and a delivery gap is only
closed at the boundary an application stands on.

Not a `spec_*` file: the spec-derived suites are an oracle written without
reading this repository. The §2.3 and §2.5 conformance statements live in
`spec_ch2_interaction.rs` and are not touched here.
