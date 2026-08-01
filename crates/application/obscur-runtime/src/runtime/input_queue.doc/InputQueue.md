The input events queued for the frame pipeline's dispatch phase, the results
that phase produced, and the frame requests it made while running.

# Why these four belong together

The last two are a latch and what it captures, and they are only correct as a
pair. While the dispatch phase runs, a `request_frame` made *by a handler* must
not reach the scheduler: the frame running the dispatch has not chosen its plan
yet, so it can serve the request itself, and routing it onward would leave a
pending request that this frame is about to satisfy — one extra full
restyle/relayout/compose per burst of input, changing nothing. So the latch
redirects those requests into a plan the running frame merges before it
decides.

Two ways to get that wrong, and both are silent: a latch left set after the
phase ends swallows every later request until something clears it, and a plan
left unmerged loses whatever the dispatch invalidated. [`begin_dispatch`] and
[`end_dispatch`] are the only ways to move the latch, and
[`take_coalesced`] empties the plan as it hands it over, so it cannot be
merged twice.

[`begin_dispatch`]: Self::begin_dispatch
[`end_dispatch`]: Self::end_dispatch
[`take_coalesced`]: Self::take_coalesced
