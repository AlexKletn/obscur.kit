Frame-phase telemetry through an installed `TelemetrySink` (spec §7.9).

What a frame *costs* is only useful if it can be attributed, and these seven
tests pin down the shape of that record. A full frame opens five spans in
pipeline order — `frame`, `reconcile`, `style`, `layout`, `compose` — and closes
four cost counters plus a `frame.phases_run` tally. The assertions compare the
whole ordered list rather than probing for one name, because a phase that
stopped recording would otherwise pass unnoticed.

Attribution is its own test, and it exists because a span that named only its
subsystem was untraceable: you could see that *layout* ran, but not which
frame's layout. Three frames over deliberately *changing* views must stamp three
distinct, increasing frame ids — and the same span leaves `node` and `resource`
empty, because a phase span is about a frame and inventing a node would be a
worse answer than none.

Two tests read the sink as an oracle for work the runtime *skipped*: three
renders of an identical view produce three `frame` spans but only **one**
`layout` span, and an idle diagnostics-capture frame opens the `frame` span
alone with `phases_run == 0`. Telemetry is the only place from outside where
"this phase did not run" is directly observable.

The budget pair is the §7.9 promise that exceeding one is a *warning*, not a
crash: a reconcile budget of zero nodes records exactly one violation, attributed
to the Tree subsystem, which converts to a `Warning` diagnostic — and a generous
budget records none. The last test is the default: with no sink installed the
frame still reconciles, still balances and still paints, and a sink swapped in
afterwards is empty, which is how "recorded nowhere observable" is proved rather
than assumed.
