The outcome a [`ShellProvider`](crate::shell::ShellProvider) returns for a
[`ShellTarget`](crate::shell::ShellTarget) (spec §6.11).

The four outcomes mirror the spec's vocabulary: the target was opened, was
denied, is unsupported, or failed. [`Denied`](Self::Denied) and
[`Unsupported`](Self::Unsupported) are *recoverable* — the app can fall back to
another strategy — whereas [`Failed`](Self::Failed) is terminal and carries a
human-readable reason.
