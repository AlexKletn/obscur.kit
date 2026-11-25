A staged, ordered batch of [`UiOperation`]s accumulated between frames.

Per spec §3.2, mutations are normalized into validated `UiOps` and committed
atomically between frames. A `UiTransaction` is the *staging* container for
those normalized ops: operations are appended in order, but **nothing is
applied** while they sit in the transaction. Application happens only after
the transaction is validated (see [`TransactionValidator`]) and committed at
a frame boundary (see [`TransactionCommit`]).

Operation order is preserved; the frame pipeline may rely on it when
interpreting the batch.

[`TransactionValidator`]: crate::transaction::TransactionValidator
[`TransactionCommit`]: crate::transaction::TransactionCommit
