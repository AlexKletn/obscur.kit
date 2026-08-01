A reference, no-op [`ShellProvider`] that opens nothing (spec §6.11).

`NullShellProvider` honestly reports that it can open no target:
[`Self::open`] returns [`ShellResult::Unsupported`] and [`Self::can_open`]
returns `false`. Its answers are correct behavior — a documented reference for
the honest-capability contract — not a stub waiting to be filled in.
