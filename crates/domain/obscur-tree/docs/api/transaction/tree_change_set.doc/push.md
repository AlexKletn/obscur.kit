Appends a change, keeping the creation counter in sync.

Internal append path used by [`TreeTransaction`](super::TreeTransaction);
callers stage changes through the transaction's typed methods rather
than pushing raw [`TreeChange`]s here.
