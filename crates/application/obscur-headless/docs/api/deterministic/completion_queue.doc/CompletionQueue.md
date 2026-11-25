An ordered queue of controlled async completions that a headless test
delivers at chosen synchronization points.

Each entry is a [`Completion<T>`] keyed by an [`OperationId`], so a test can
resolve async results deterministically and callers can drop stale results
whose operation no longer matches (spec §3.3, §7.4). Completions are
delivered in strict FIFO order.
