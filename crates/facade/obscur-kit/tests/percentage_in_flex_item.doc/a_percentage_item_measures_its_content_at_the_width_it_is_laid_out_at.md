The item is 200 (half of 400) and 60 tall — one line.

Before the fix this read 200 x **120**: the item's cross measure was handed
its own used main size as the only number available, resolved `width: 50%`
against it, laid its content out at 100, and reported the two wrapped lines
that width forces.
