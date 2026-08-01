Returns whether pending background work has been starved and must be
serviced next (§3.6).

This is `true` only when background work is pending **and** the starvation
guard reports it as starved — that is, higher-priority frames have
preempted it past the fairness threshold. When this returns `true` the
runtime should run the background work and call
[`service_background`](FrameScheduler::service_background).
