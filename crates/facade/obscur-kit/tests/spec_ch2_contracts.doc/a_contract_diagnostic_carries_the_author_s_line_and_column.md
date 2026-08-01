§2.15: "- source-mapped;" (with §2.8 "- stable source mapping;")

WEAKENED 2026-07-19: this originally demanded the diagnostic report LINE 12,
COLUMN 5. The two clauses cited are the bullets "- stable source mapping;"
(a goal of the Rust DSL) and "- source-mapped;" (a property of UI IR);
neither says line-and-column, and the front end reports a byte offset that
resolves to exactly the right place in the authored text. The assertion is
now that the location is the author's offending construct rather than a
macro-expansion site or the start of the file — the same defect the case was
written to catch, stated in the units the implementation reports. Candidate
spec amendment: fix the presentation of a source location (offset vs
line/column) so tooling can depend on it.
