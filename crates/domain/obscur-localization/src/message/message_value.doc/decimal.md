Binds a number written as `decimal` (`"1"`, `"1.0"`, `"2.50"`).

The spelling is the input because the trailing zeros change the category —
see [`PluralOperands::from_decimal`], which is also why this returns `None`
for a spelling it cannot read.
