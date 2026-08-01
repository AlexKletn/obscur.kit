Ends the drag as the pointer is released.

A drop over an accepting target **commits** the staged work: it is
validated and returned as a [`TransactionCommit`] for the caller to apply.
Anything else **cancels**: the staged work is discarded and nothing is
applied.

A commit whose validation fails also cancels rather than half-applying —
which is the atomicity [`ProvisionalTransaction::commit`] exists to give.

Returns [`None`] when no drag was in flight.
