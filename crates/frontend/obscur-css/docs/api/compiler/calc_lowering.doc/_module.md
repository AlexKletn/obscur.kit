Lowering `calc()` / `min()` / `max()` / `clamp()` into a symbolic
[`LengthExpr`] (spec §4.13).

The value parser keeps math functions as a raw component list — operands
interleaved with [`ParsedValue::Delimiter`] operators and
[`ParsedValue::Comma`] separators (see [`crate::parser::value_parser`]). This
module folds that list into an [`LengthExpr`] tree that resolves to concrete
pixels later, at the layout/compute step, against a `LengthContext`. Nothing
is evaluated here: the font size and viewport are still unknown, so the
expression stays symbolic.

Grammar (per math function argument):

```text
sum     := product (('+' | '-') product)*
product := unit    (('*' | '/') unit)*
unit    := <length> | <number> | '(' sum ')' | <math-fn>
```

`*` and `/` follow the CSS rule that exactly one operand is a plain number;
the coefficient is carried on [`LengthExpr::Product`] / [`LengthExpr::Quotient`].
