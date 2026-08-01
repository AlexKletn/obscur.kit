**SPECIFICATION-DERIVED** window, activation, presentation, chrome and
display cases for obscur.kit §6.1-§6.6.

Read this before touching anything in the file.

Every expected value below was produced by an agent reading obscur.kit §6
with **no access to this repository's source**. The cases are a statement of
what the implementation owes the spec, not a record of what the
implementation currently does. A failure here is a defect report about the
toolkit (or, occasionally, about the spec), and must not be "corrected" by
editing the expected value.

The subject under test is [`NullWindowBackend`] — the kit's own reference
headless [`WindowBackend`], the backend an application gets in §7.4 "Official
headless mode operates without window or GPU." It is kit code, not a mock
written for these tests.

Each test's doc comment carries the quoted normative sentence it encodes.
