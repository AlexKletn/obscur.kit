The control's **children** — the circular indicator sub-part followed by the
label [`Text`] — derived from the only two values that decide them: whether
the radio is selected, and the label text.

It is an associated function rather than a method because it has two callers
that hold those values in different places, and they must agree exactly:

- [`view`](Self::view) wraps them in the widget's own root element (the
  Rust-API entry point);
- [`RadioComponent`](super::RadioComponent) returns them *as* its render
  output, because there the root is the authored `<radio>` node the runtime
  already put in the tree.

Keeping one body means a mounted radio and a built one can never draw
different indicators. `disabled` is absent for the same reason it is absent
from [`Checkbox::contents`]: it changes the cascade (`:disabled`), never the
children — the UA sheet's `radio:disabled .radio-dot` rule is what makes a
disabled radio look disabled.

[`Text`]: obscur_primitives::Text
[`Checkbox::contents`]: crate::checkbox::Checkbox
