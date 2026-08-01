A literal value carried by a [`UiIrExpression::Literal`] (spec §2.5).

This mirrors the runtime `ViewValue`: the same closed set of scalar kinds a
property can hold, but as the serializable compile artifact rather than the
live desired-tree value. Because the [`Float`](UiIrValue::Float) variant
carries an `f64`, this enum derives only `Debug`, `Clone`, and `PartialEq`
— not `Eq`/`Hash` (floats are not totally ordered and have `NaN`).
