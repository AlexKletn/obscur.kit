Opens `target` through the shell and returns the typed outcome
(spec §6.11).

A provider that cannot open this target returns
[`ShellResult::Unsupported`] rather than panicking.
