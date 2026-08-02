The decoded properties of an `<input>` or a `<textarea>`: the field's text,
and nothing else.

This is the hand-written equivalent of what `#[derive(Contract)]` emits, for
the reason the other built-ins are hand-written too: the derive expands to
paths rooted at `::obscur_kit::…` and crate-architecture §3.16 forbids an
internal crate from depending on the facade, so a shipped widget cannot
derive its own contract. [`contract`](Self::contract) is byte-for-byte the
contract the derive would produce for
`#[contract(name = "Input")] struct { #[contract(default)] value: String }`,
and `obscur-kit`'s `builtin_components` test proves that by comparing the two
values.

**One field, and four things that are deliberately not fields.**

- `placeholder` is a §4.16 *synthetic part*: the runtime paints it from the
  reserved `PLACEHOLDER` attribute on the host node while the value is empty,
  and it is never a tree node. A component renders its host's **children**,
  so a placeholder field here would be a value this component could not act
  on.
- `readonly` and `disabled` are reserved structural attributes, read straight
  off the node — by the cascade for `:disabled`, and by the runtime's editing
  session for the mutation gate. They change how the control behaves and how
  it looks, never what it renders.
- **`multiline` is not a prop at all**: `<input>` and `<textarea>` share one
  [`ComponentId`], and which of them a node is rides on its *local name*,
  which is what `EditableField::is_multiline` already reads. The host node
  keeps its own props across a component render, so mounting preserves the
  distinction by not touching it.

[`ComponentId`]: obscur_identity::ComponentId
