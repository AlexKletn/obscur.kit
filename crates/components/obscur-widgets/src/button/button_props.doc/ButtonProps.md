The decoded properties of a `<button>`: its label, and only its label.

This is the hand-written equivalent of what `#[derive(Contract)]` emits — the
derive expands to paths rooted at `::obscur_kit::…` and crate-architecture
§3.16 forbids an internal crate from depending on the facade, so a shipped
widget cannot derive its own contract. Two of its answers are deliberately
**not** the ones the derive would give, and both are the same fact seen twice:

1. **`label` is defaulted, not required.** The derive makes a `String` field
   required, and the `.okui` compiler does require it of the `<Button/>`
   *component* spelling. The base-element spelling states the same text as
   authored content — `<button>Save</button>` carries no `label` prop at all —
   and a required prop would fail to decode on it, which would replace the
   author's own markup with an error boundary.
2. **The contract declares the default slot** ([`contract`](Self::contract)),
   which no derive attribute can spell: `#[contract(slot(name = "…"))]` names
   a slot, and the one this control has is the unnamed one.

`disabled`, `pressed` and the click binding are deliberately not fields. They
are reserved structural attributes and an event binding on the host node,
read by the cascade and by the runtime's click resolver; a mounted component
renders its host's *children* and never its host's props, so decoding them
here would be values with no reader.
