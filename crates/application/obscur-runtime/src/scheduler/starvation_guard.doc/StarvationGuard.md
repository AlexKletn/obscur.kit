A deterministic fairness counter that prevents lower-priority background work
from being starved (spec §3.6 "starvation prevention for background work").

This is **not** time-based: it carries no wall-clock and observes no timers.
Instead it counts how many times higher-priority frames have been served
while a piece of background/low-priority work waited. Once that count reaches
a threshold the guard reports the work as starved, letting the scheduler
force a promotion so it finally runs.
