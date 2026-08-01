Records a normalized platform `input` at `tick`, applying the redaction
policy for its `sensitivity`.

When the policy redacts that class, only a [`RecordedEvent::Redacted`]
marker (carrying the [`Sensitivity`] class) is stored — the original
`input` payload is dropped and never persisted (spec §7.13). A
[`Sensitivity::Normal`] input, or any class the policy does not redact,
is recorded verbatim as [`RecordedEvent::Input`].
