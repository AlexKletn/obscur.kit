§2.10, Forbidden: "- async work;"

WEAKENED 2026-07-19: this originally required the diagnostic to contain the
word "async" (`reports(source, "async")`). The await IS refused; §2.10 lists
forbidden forms and imposes no requirement on the message. Candidate spec
amendment: as for `assignment_in_an_expression_is_rejected` — say whether a
§2.10 refusal must attribute itself to a named category.
