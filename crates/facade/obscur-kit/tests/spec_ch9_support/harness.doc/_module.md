Shared harness for the `spec_ch9_*` **specification-derived** test files.

These tests are NOT implementation-derived. Every expected value in the
`spec_ch9_*.rs` files was written by an agent reading obscur.kit §9
("Roadmap & non-goals") together with the §1–§8 clause that turns each
non-goal into a normative rule, **without opening any source in this
repository**. This module exists only to give those cases a compiling home;
it contains no expectations of its own.

Nothing here may be "fixed" to make a case pass. A failing `spec_ch9_*` test
is a claim about the implementation, not about the test.
