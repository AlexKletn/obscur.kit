A tally of what happened while replaying an
[`EventRecording`](super::EventRecording).

Counts are cumulative over the whole replay: how many steps were walked, how
many redaction markers were skipped (their payloads were dropped at record
time and can never be replayed, spec §7.13), how many inputs were applied,
how far the deterministic clock was advanced, and how many async completions
were dispatched — plus the final [`ReplayStatus`].
