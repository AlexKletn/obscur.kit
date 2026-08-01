Whether flex items are confined to a single line or wrapped onto multiple
lines along the cross axis (spec §4.3).

[`NoWrap`](FlexWrap::NoWrap) (the default) keeps every item on one main-axis
line, shrinking/overflowing as needed. [`Wrap`](FlexWrap::Wrap) breaks items
onto successive lines that stack down the cross axis;
[`WrapReverse`](FlexWrap::WrapReverse) stacks those lines in the opposite
cross-axis order.
