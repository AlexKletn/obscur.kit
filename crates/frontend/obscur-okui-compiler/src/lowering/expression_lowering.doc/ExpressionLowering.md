Lowers a `.okui` AST [`Expression`] (raw interpolation text) into a
[`UiIrExpression`] (spec §2.10).

# Classification

This is a **minimal literal-vs-binding classifier**, not a full expression
parser. The trimmed expression text is matched, in order, against:

- `true` / `false` → a boolean [`literal`](UiIrExpression::literal);
- an integer (`[+-]?[0-9]+`) → an integer literal;
- a decimal containing a `.` → a float literal;
- a double-quoted `"…"` string → a text literal (quotes stripped);
- `null`, `none` or empty text → a [`Null`](UiIrValue::Null) literal;
- anything else (an identifier or dotted path such as `user.name`) → a
  [`binding`](UiIrExpression::binding).

Parsing arbitrary operators or producing [`Call`](UiIrExpression::Call)
nodes is future work; such input falls through to a binding on its raw text.
