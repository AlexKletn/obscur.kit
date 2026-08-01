`justify-self: start` was the audit's own proof that the declared value is
understood and the *alignment* was what discarded it: it already gave 50 while
the default gave 211. It still gives 50 — the fallback replaced `stretch`, and
left every other alignment keyword exactly where it was.
