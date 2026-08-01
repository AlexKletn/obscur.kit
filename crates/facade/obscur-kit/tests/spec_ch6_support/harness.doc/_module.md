Shared harness for the `spec_ch6_*` **specification-derived** test files.

These tests are NOT implementation-derived. Every expected value in the
`spec_ch6_*.rs` files was written by an agent reading obscur.kit §6
(Platform, windows & services) **without opening any source in this
repository**. This module exists only to give those cases a compiling home;
it contains no expectations of its own.

Nothing here may be "fixed" to make a case pass. A failing `spec_ch6_*` test
is a claim about the implementation, not about the test.
