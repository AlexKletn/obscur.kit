§2.10 Expressions — "Forbidden:" / "- assignment;" and §2.9 "- no business
logic."

NG-OKUI-ASSIGN-01.

WEAKENED 2026-07-19: this also asserted `result.document().is_none()` — that
a compile with errors emits NO output artifact. §2.10 lists forbidden
expression forms and §2.13 says unknown attributes, events and string method
names "are compile errors"; neither says an erroring compile must suppress
its artifact, and a front end that reports errors while still handing back a
partial IR (so an IDE can keep offering completion and an incremental build
can keep a stale-but-typed tree) is a defensible design. The refusal itself
is kept and is what discriminates. Candidate spec amendment: state whether
`has_errors()` implies no artifact, since a caller that checks only
`document().is_some()` currently ships a document built from refused input.
