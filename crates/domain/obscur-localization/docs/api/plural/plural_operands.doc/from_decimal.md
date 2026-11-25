The operands of a number as it is *written*, parsed from its decimal
spelling.

The spelling is the input because the trailing zeros are the input:
`"1"`, `"1.0"` and `"1.00"` are one value and three distinct operand sets.
Accepts an optional sign, digits, and an optional `.` fraction; returns
`None` for anything else (an exponent, a grouping separator, an empty
fraction, a bare `.`).

A fraction longer than 19 digits, or an integer part beyond [`u64::MAX`],
is refused rather than truncated into a different category.
