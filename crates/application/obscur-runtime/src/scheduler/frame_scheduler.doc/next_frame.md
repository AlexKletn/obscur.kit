Pops the most-urgent pending frame to run, or [`None`] if idle (§3.6).

The frame is drawn from the queue in priority order. If a frame is
returned while background work is pending, that frame has preempted the
waiting background work, so the starvation guard records a preemption
([`StarvationGuard::record_preempted`]) — moving the background work one
step closer to being reported as starved
([`background_starved`](FrameScheduler::background_starved)).

When the queue is empty this returns [`None`] without side effects: the
scheduler is event-driven and does not busy-loop (§3.5).
