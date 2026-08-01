Shared harness for the `spec_ch2_*` **specification-derived** test files.

These tests are NOT implementation-derived. Every expected value in the
`spec_ch2_*.rs` files was written by an agent reading obscur.kit §2
(*Components, tree & DSL*) **without opening any source in this repository**.
The cases are therefore a statement of what the implementation owes the
spec, not a record of what it currently does.

This module exists only to give those cases a compiling home; it contains no
expectations of its own. Nothing here may be "fixed" to make a case pass. A
failing `spec_ch2_*` test is a claim about the implementation, not about the
test.
