Where a [`ContractViolation`] should be reported, as an index into the
[`ElementUse`] that produced it.

The rule does not know what a location *is* — the file path has byte offsets
and the macro has spans — so it names the offending part and lets the caller
resolve it.
