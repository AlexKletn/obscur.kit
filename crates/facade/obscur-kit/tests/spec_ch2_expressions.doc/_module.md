**SPECIFICATION-DERIVED** expression and `.okui` front-end cases for
obscur.kit §2.9 (*External `.okui` files*), §2.10 (*Expressions*) and §2.11
(*Conditions and collections*).

Read this before touching anything in the file.

Every expectation below was produced by an agent reading obscur.kit §2 with
**no access to this repository's source**. The cases are a statement of what
the implementation owes the spec, not a record of what it currently does. A
failure here is a defect report about the engine (or, occasionally, about the
spec) and must not be "corrected" by editing the expected value.

# Why the file front end and not the macro

§2.10 is a set of *rejections*. A rejection expressed through the `okui!`
macro is a compile error in this very file, so the test binary would not
build and would report nothing. Every rejection case therefore drives the
`.okui` string path, whose verdict is a value (`CompileResult`) rather than a
compiler error — the same rule, reached through the front end that can be
interrogated.

Each test's doc comment carries the quoted normative sentence it encodes.
