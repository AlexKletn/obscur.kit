An ordered, timestamped log of recorded steps together with the
[`RedactionPolicy`] under which it was captured (spec §7.13).

Steps are appended in order via [`record`](Self::record) /
[`record_input`](Self::record_input). Recording an input goes through the
policy: a sensitive class that the policy redacts is stored as a
[`RecordedEvent::Redacted`] marker with its payload dropped, so a recording
never carries redacted secrets.
