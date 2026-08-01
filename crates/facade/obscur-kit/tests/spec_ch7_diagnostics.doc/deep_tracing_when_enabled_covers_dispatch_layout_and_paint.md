§7.9, verbatim: "Deep tracing is opt-in. Lightweight counters may remain
available in production only when explicitly enabled." with "Spans and
counters may cover:" / "- dispatch;" / "- layout;" / "- paint;".

FIXTURE REPAIRED and then WEAKENED 2026-07-19. Two separate faults.

(a) The body was ten bare `render_now` calls and NO INPUT AT ALL, so no
dispatch span could exist under any implementation — the case demanded
coverage of a phase it never caused to run, and its failure — `spans cover
["dispatch"] at minimum` — read as a missing-instrumentation bug. Real input
is now dispatched before each frame after the first.

(b) With the fixture repaired, no dispatch span appears — and §7.9 does not
require one. Its list opens "Spans and counters MAY cover:" and dispatch is
one of thirteen bullets under that permission; the only normative sentences
are "Deep tracing is opt-in." and "Lightweight counters may remain available
in production only when explicitly enabled." So `dispatch` is dropped from
the mandatory-coverage loop.

The opt-in sentence itself is NOT asserted here, deliberately: with no sink
installed there is no collector to read, so "collected nothing" is
unobservable from outside and any assertion of it would be vacuously true.
Recorded rather than faked.

Candidate spec amendment: turn §7.9's "may cover" list into a required
minimum (dispatch, layout, paint at least) if deep tracing is meant to be
usable for diagnosing input latency — today an implementation may instrument
nothing at all and conform.
