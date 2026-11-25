`filter` and `backdrop-filter` lowering: an ordered [`FilterOp`] chain
(Filter Effects 1 §5, §7).

The chain is the entity: each authored function is one operation, the list
order is the application order, and an unrecognised function is dropped
rather than failing the declaration — so the vocabulary of accepted names is
what this file *is*. `drop-shadow()` borrows the `<shadow>` grammar from
[`super::shadow_lowering`]; everything else here (the colour-matrix family,
the amount and angle argument readers, the blur radius) is used by no other
property.
