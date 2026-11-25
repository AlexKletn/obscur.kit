A single `.counter` element, 100×50 with a solid red fill.

This is the smallest view that still exercises every phase: it reconciles to one
node, matches one rule, arranges one box and composes one fill — so a test that
wants to talk about *frames* rather than about layout can say `CounterPanel` and
stop thinking about the tree. Nearly every scheduling, transaction, telemetry and
motion assertion in the suite is written against its 100×50 box.

[`nested_view`](Self::nested_view) is the same element with one anonymous child,
and exists for exactly one question: *did the reconcile phase run?* Against an
identical view a reconcile commits empty and `reconciled()` is false either way,
so the assertion would pass without the plan doing anything at all.
