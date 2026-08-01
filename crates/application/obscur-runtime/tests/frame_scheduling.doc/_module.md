When a frame runs, and what a transaction bought when it asked for one
(spec §3.2, §3.4, §3.6).

Nine tests about the *outside* of the frame pipeline: a request goes in, a frame
comes out, and the questions are how many, in what order, and carrying which
work. Two halves that turn out to be one subject — the scheduler decides
**whether** a frame runs, and a committed transaction decides **what** the frame
that runs recalculates — because the failure they both guard against is a frame
that costs its budget and does nothing with it, or a request that quietly buys
nothing at all.

The background band is here for the same reason. `request_background` is not a
frame request; it is a promise that low-priority work will eventually be served
*despite* a stream of higher-priority input frames, and §3.6's starvation guard
is what keeps that promise. Proving it on the real `run_due_frame` drain path
rather than on a bare `FrameScheduler` is the whole point of those two tests.
