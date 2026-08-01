§2.10, Forbidden: "- I/O;" and "- direct service access;"

WEAKENED 2026-07-19, I/O half only: this originally required the I/O
diagnostic to contain the string "I/O" (`reports(io, "I/O")`). The call IS
refused — as an undeclared free call — and §2.10 says nothing about
attributing a refusal to its category. Candidate spec amendment: state
whether a §2.10 refusal must name the category it violated.

The direct-service-access half is DELIBERATELY LEFT INTACT and is expected to
fail: `{services.http.get(url)}` compiles clean whenever `services` is a
declared prop, so the "- direct service access;" bullet is genuinely
unenforced. That is an implementation gap, not a test defect.
