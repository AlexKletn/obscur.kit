Shared harness for the `spec_ch4b_*` **specification-derived** test files.

These tests are NOT implementation-derived. Every expected value in the
`spec_ch4b_*.rs` files was derived from the obscur.kit specification's §4
clauses (the sections an earlier pass did not cover). This module exists only
to give those cases a compiling home; it holds no expectations of its own.

Nothing here may be "fixed" to make a case pass. A failing `spec_ch4b_*` test
is a claim about the implementation, not about the test.
