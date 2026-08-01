Whether the caller can continue after this outcome.

A successful present, a skipped stale frame, and a device-loss event are
all recoverable: the caller can render again. Only [`PresentResult::Failed`]
is terminal (spec §5.2).
