How sensitive a recorded input is — the classification that drives redaction.

A caller tags each recorded input with its sensitivity so the
[`RedactionPolicy`](super::RedactionPolicy) can decide whether the payload is stored verbatim or
dropped and replaced with a [`RecordedEvent::Redacted`] marker (spec §7.13).
