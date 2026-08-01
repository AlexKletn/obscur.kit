§2.11: "- mandatory stable `key`;" (the duplicate half — silently losing a
row is far worse than a diagnostic)

WEAKENED 2026-07-19: this originally also asserted that a Diagnostic naming
the duplicated key reaches the runtime's sink. §2.11's whole text here is
"- mandatory stable `key`;" — mandatory is enforced (see
`a_for_without_a_key_is_a_compile_error`), and nothing in §2.11 or §1.6
requires a duplicate-key diagnostic. The surviving assertion is the one that
matters: the list is not silently collapsed. Candidate spec amendment: say
whether duplicate key values are a diagnosable fault, and at which layer
(compile time or reconciliation).
