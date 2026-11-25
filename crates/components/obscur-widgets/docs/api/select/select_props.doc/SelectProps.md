The decoded properties of a `<select>`: the option labels, which one is
selected, and whether the popup is down.

**A select's options are a prop, not projected children**, and the reason is
what [`Select::with_options`] already takes: a sequence of *labels* —
`Into<Box<str>>` — not a sequence of views. The control synthesises each row
itself, because each row is more than its text: a `.select-option` element
with a per-index [`ComponentId`], and a `selected` class the control computes
from the current selection. Projecting authored children as options would hand
the author a slot they could put an `<img>` in, and then require the control to
reduce that back to the `Box<str>` its model holds — a reduction that is lossy
where it is possible at all. The header, the arrow, the list and the rows are
the control's; only the labels are the author's, and a label is a string.

This is the hand-written equivalent of what `#[derive(Contract)]` emits, for
the same two reasons [`CheckboxProps`] is hand-written — the derive expands to
`::obscur_kit::…` paths a components-tier crate may not name (crate-arch
§3.16), and it could not express this decoding anyway — plus a third that is
this control's own:

**The contract records the type of the *authored* value, and `options` is
authored as a string.** A [`ViewValue`] has no list variant, so there is no
`Vec<Box<str>>` on a node to decode; there is a delimited string, which this
struct turns into the list the control renders from. `contract()` is therefore
byte-for-byte the contract the derive would produce for
`#[contract(name = "Select")] struct { options: Option<String>, selected:
Option<usize>, #[contract(default)] open: bool }` — the wire shape — and
`obscur-kit`'s `builtin_components` test proves that by comparing the two
values. Only `options` differs from the field beside it; `selected` and `open`
are the same types here as there.

`disabled` is deliberately **not** a field or a contract prop: it is a
reserved structural attribute, like `class` and `id`, read straight off the
node by the cascade. It changes how the control looks, never what it renders —
which is exactly why [`Select::contents`] takes `open` and not `disabled`, and
why an open *and* disabled select is a state this type can hold.

[`Select::with_options`]: super::Select::with_options
[`Select::contents`]: super::Select
[`CheckboxProps`]: crate::checkbox::CheckboxProps
[`ComponentId`]: obscur_identity::ComponentId
[`ViewValue`]: obscur_view::view::ViewValue
