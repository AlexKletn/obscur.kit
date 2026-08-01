Returns the ordered `filter` op list if this value is a
[`PropertyValue::Filter`] (an empty slice for `filter: none` or a filter
with no honored functions). Each entry is a [`FilterOp`]
(a `drop-shadow(...)` or a `blur(<length>)`), in author order.
