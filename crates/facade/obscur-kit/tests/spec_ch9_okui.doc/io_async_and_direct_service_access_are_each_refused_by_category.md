§2.10 — "Forbidden:" / "- I/O;" / "- async work;" / "- direct service
access;" and §9.1 "- automatic execution of business logic inside `.okui`."

NG-OKUI-IO-01.

WEAKENED 2026-07-19: this required each refusal's message to NAME ITS
CATEGORY — "i/o", "async", "service". §2.10 is a list of forbidden *things*,
not a list of required *messages*: it nowhere says a refusal must be
attributed to the category it violated, and §1.6's diagnostic-shape rule asks
for severity, subsystem, stable code, context and recovery action, not a
category word in the prose. All three forms are in fact refused. Candidate
spec amendment: decide whether a §2.10 refusal must classify itself — the
same amendment the ch2 twins of these cases ask for, and worth answering
once, since a single generic "not a valid expression" message is what an
author sees for six quite different mistakes.

The `services` line now DECLARES `services` as a prop, which is the
configuration that matters: an undeclared `services` is refused as an
unresolved name, which would make this line pass without testing
"- direct service access;" at all. Declared, it is the honest test — and it
is expected to fail, because a declared `services` compiles clean. That is
the registered real gap, now isolated where it belongs.
