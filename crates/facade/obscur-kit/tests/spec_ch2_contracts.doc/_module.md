**SPECIFICATION-DERIVED** component-contract, template and slot cases for
obscur.kit §2.8 (*Rust DSL*), §2.12 (*Templates and slots*) and §2.13
(*Component contracts*).

Read this before touching anything in the file.

Every expectation below was produced by an agent reading obscur.kit §2 with
**no access to this repository's source**. The cases are a statement of what
the implementation owes the spec, not a record of what it currently does. A
failure here is a defect report about the engine (or, occasionally, about the
spec) and must not be "corrected" by editing the expected value.

Rejection cases drive the `.okui` string path rather than the `okui!` macro,
for the reason spelled out in `spec_ch2_expressions.rs`: a rejection the
macro reaches is a compile error in this file, and a test binary that does
not build reports nothing.

Each test's doc comment carries the quoted normative sentence it encodes.
