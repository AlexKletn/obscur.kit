The control's **children** — the boxed indicator sub-part followed by the
label [`Text`] — derived from the only two values that decide them: the
tri-state `checked` (`None` = mixed) and the label text.

It is an associated function rather than a method because it has two
callers that hold those values in different places, and they must agree
exactly:

- [`view`](Self::view) wraps them in the widget's own root element (the
  Rust-API entry point);
- [`CheckboxComponent`](super::CheckboxComponent) returns them *as* its
  render output, because there the root is the authored `<checkbox>` node
  the runtime already put in the tree.

Keeping one body means a mounted checkbox and a built one can never draw
different indicators. See [`contents`](Self::contents)' callers for why
`disabled` is absent: it changes the cascade (`:disabled`), never the
children.

[`Text`]: obscur_primitives::Text
