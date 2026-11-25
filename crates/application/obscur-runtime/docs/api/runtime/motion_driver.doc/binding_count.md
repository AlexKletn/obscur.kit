How many bindings are held, running or settled.

Test-only: no shipped surface reads it. It exists to pin the behaviour the
[`settled_progress`](Self::settled_progress) field is *for* — a finished
timeline is pruned from the scheduler while its binding lives on to hold
the endpoint value — which no public counter can distinguish, because both
public counters report the scheduler's active timelines.
