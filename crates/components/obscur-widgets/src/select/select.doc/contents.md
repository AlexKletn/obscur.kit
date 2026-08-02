The control's **children** — the `.select-header` row (current label +
disclosure arrow) and, while open, the `.select-list` popup — derived from the
only three values that decide them: the option labels, which one is selected,
and whether the popup is down.

It is an associated function rather than a method because it has two callers
that hold those values in different places, and they must agree exactly:

- [`view`](Self::view) wraps them in the widget's own root element (the
  Rust-API entry point);
- [`SelectComponent`](super::SelectComponent) returns them *as* its render
  output, because there the root is the authored `<select>` node the runtime
  already put in the tree.

**`disabled` is not a parameter, and that is load-bearing rather than
incidental.** A disabled select renders exactly what an enabled one renders;
the difference is entirely the cascade's (`select:disabled`,
`select:disabled .select-arrow`, `select:disabled .select-option`). In
particular this function draws the popup from `open` **alone** — it does not
re-apply [`open`](Self::open)'s refusal to open a disabled control. That
refusal is a rule about a *transition*, and re-applying it to a *state* would
delete the one state in which the UA sheet's `select:disabled .select-option`
rule can match: a control disabled while its popup was already down.
