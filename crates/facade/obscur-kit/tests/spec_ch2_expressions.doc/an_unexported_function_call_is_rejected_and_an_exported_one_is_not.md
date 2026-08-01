§2.10, Forbidden: "- unconstrained arbitrary function calls;"

The pair is the whole point: `fmt_price` is an explicitly exported pure
function for templates and must be accepted; `fmt_tax` is an ordinary
function with no such marking and must be refused. Only asserting both in
one file catches either error.

WEAKENED 2026-07-19, `fmt_tax` half only: this originally asserted that some
diagnostic message *names the callee* (`m.contains("fmt_tax")`) and that no
message names `fmt_price`. §2.10 forbids "- unconstrained arbitrary function
calls;" and requires no wording; both calls are in fact refused, with the
front end's generic §2.10 message and correct spans. Candidate spec
amendment: state whether a rejected call's diagnostic must identify the
callee (and whether spans alone suffice).

The `fmt_price` half is DELIBERATELY LEFT INTACT and is expected to fail:
§2.10's Allowed list ends with "- explicitly exported pure functions." and
there is no declaration surface that makes any call legal. That is an
implementation gap, not a test defect.
