Creates a guard that reports starvation after `threshold` preemptions.

A `threshold` of `0` disables the guard: it never reports starvation (see
[`is_starved`](StarvationGuard::is_starved)).
