Shared harness for the `spec_ch7b_*` **specification-derived** test files.

These tests are NOT implementation-derived. Every expected value in the
`spec_ch7b_*.rs` files was written by an agent reading the specification
(obscur.kit §7.11 plus the README's core principles) rather than the source.
This module exists only to give those cases a compiling home; it **drives**
and it **records**, and it carries no expectations of its own.

Nothing here may be "fixed" to make a case pass. A failing `spec_ch7b_*` test
is a claim about the implementation, not about the test.
