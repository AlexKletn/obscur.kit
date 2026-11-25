The decoded properties of a `<checkbox>`: the label text and the tri-state
checked value.

This is the hand-written equivalent of what `#[derive(Contract)]` emits,
and it is hand-written for two reasons that a reader should not have to
rediscover:

1. **The derive is out of reach.** It expands to paths rooted at
   `::obscur_kit::…`, and crate-architecture §3.16 forbids an internal
   crate from depending on the facade. A shipped widget therefore cannot
   derive its own contract. `contract()` below is byte-for-byte the
   contract the derive would produce for
   `#[contract(name = "Checkbox")] struct { label: String, checked:
   Option<bool> }`, and `obscur-kit`'s `builtin_components` test proves
   that by comparing the two values.
2. **The derive could not express the decoding even if it were reachable.**
   See [`from_props`](Self::from_props): `checked` is a *reserved
   structural attribute* whose presence — not its value — sets the state
   the cascade reads for `:checked`, and the widget additionally has a
   third (mixed) state that no boolean carries. The derive's
   `Option<bool>` field decodes "absent" and "present but null" alike to
   `None`, which would silently disagree with the `:checked` the same node
   matches.

`disabled` is deliberately **not** a field or a contract prop: it is a
reserved structural attribute, like `class` and `id`, read straight off
the node by the cascade. It changes how the control looks, never what it
renders, so decoding it here would be a value with no reader.
