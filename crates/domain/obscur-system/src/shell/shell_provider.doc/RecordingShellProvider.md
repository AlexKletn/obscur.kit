A deterministic, recording [`ShellProvider`] for tests (spec §6.11).

`RecordingShellProvider` accepts every target: [`Self::open`] pushes the
target onto [`Self::opened`] and returns [`ShellResult::Opened`], and
[`Self::can_open`] is always `true`. Tests inspect the recorded targets to
assert what the domain attempted to open, without touching a real shell.
