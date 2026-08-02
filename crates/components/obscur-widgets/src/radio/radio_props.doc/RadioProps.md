The decoded properties of a `<radio>`: the label text and whether the radio
is selected.

This is the hand-written equivalent of what `#[derive(Contract)]` emits, for
the same two reasons [`CheckboxProps`] is hand-written:

1. **The derive is out of reach.** It expands to paths rooted at
   `::obscur_kit::…`, and crate-architecture §3.16 forbids an internal crate
   from depending on the facade. A shipped widget therefore cannot derive its
   own contract. `contract()` below is byte-for-byte the contract the derive
   would produce for
   `#[contract(name = "Radio")] struct { label: String,
   #[contract(default, name = "checked")] selected: bool }`, and
   `obscur-kit`'s `builtin_components` test proves that by comparing the two
   values.
2. **The derive could not express the decoding even if it were reachable.**
   See [`from_props`](Self::from_props): the authored spelling is the
   *reserved structural attribute* `checked`, whose presence — not its value —
   sets the state, while the derive's `bool` field would read a valueless
   attribute as absent.

**The authored attribute is `checked`, not `selected`.** `selected` is the
name of the builder's method ([`Radio::selected`]); `checked` is the name of
the thing on the node. They are the same fact, and only one of the two
spellings is also what the cascade reads for `:checked` — so a radio written
`<radio checked/>` cannot paint a filled dot while `radio:checked` fails to
match it. Giving markup the other spelling would have bought a second name
for one state at the price of exactly that disagreement.

`disabled` is deliberately **not** a field or a contract prop: it is a
reserved structural attribute, like `class` and `id`, read straight off the
node by the cascade. It changes how the control looks (`radio:disabled
.radio-dot`), never what it renders, so decoding it here would be a value
with no reader.

[`CheckboxProps`]: crate::checkbox::CheckboxProps
[`Radio::selected`]: super::Radio::selected
