A deterministic replayer for an [`EventRecording`].

[`replay`](Self::replay) walks the recording's fixed step list in order and
dispatches each step to a [`ReplayTarget`]. Because the step list is fixed and
dispatch is a pure walk, replaying the same recording into equivalent targets
always produces the same sequence of calls — the determinism record-replay
relies on (spec §7.13). Redaction markers carry no payload and are skipped
(but counted).
