Replays every recorded step in order into `target`, returning a tally.

Each step is dispatched to the matching [`ReplayTarget`] method;
[`Redacted`](RecordedEvent::Redacted) markers are skipped and counted
(their payloads were dropped at record time, spec §7.13). The returned
[`ReplayResult`] always has [`ReplayStatus::Completed`] — a plain replay
of a fixed step list cannot diverge; use
[`replay_checked`](Self::replay_checked) to detect divergence against an
expected recording.
