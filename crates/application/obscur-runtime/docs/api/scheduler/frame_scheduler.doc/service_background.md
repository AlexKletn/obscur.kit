Records that the pending background work was run, resetting fairness.

Calling this resets the starvation guard
([`StarvationGuard::record_served`]) so the fairness count starts over.
The runtime **should** call this when
[`background_starved`](FrameScheduler::background_starved) returns `true`,
servicing the background work next to honor §3.6 starvation prevention.
