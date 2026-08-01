Shared harness for the `spec_ch5b_*` **specification-derived** test files.

These tests are NOT implementation-derived. Every expected value in the
`spec_ch5b_*.rs` files was written by an agent reading the specification
(obscur.kit §5.4 "Overlay host mode", §5.5, §5.9, §5.13) **without opening
any source in this repository**. This module exists only to give those cases
a compiling home; it contains no expectations of its own.

Nothing here may be "fixed" to make a case pass. A failing `spec_ch5b_*` test
is a claim about the implementation, not about the test.
