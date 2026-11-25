Records that background task work is now waiting to be serviced.

The runtime calls this when subsystem shaping / decode / cache work
(§3.6 band 5) has become pending; while set, frames served by
[`next_frame`](FrameScheduler::next_frame) count as preemptions of that
work.
