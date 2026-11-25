Returns whether the outcome is recoverable — the app may fall back to
another strategy (spec §6.11).

[`Denied`](Self::Denied) and [`Unsupported`](Self::Unsupported) are
recoverable; [`Failed`](Self::Failed) is terminal and
[`Opened`](Self::Opened) needs no recovery.
