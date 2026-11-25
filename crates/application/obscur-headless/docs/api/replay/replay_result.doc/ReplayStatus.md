Whether a replay ran to completion or diverged from an expected trace.

Basic replay always yields [`ReplayStatus::Completed`] — it walks the fixed,
recorded step list and dispatches each step, which cannot diverge. Divergence
is only reported by the optional checked replay
([`ReplaySession::replay_checked`](super::ReplaySession::replay_checked)),
which flags the first step whose observed effect differs from an expected
recording.
