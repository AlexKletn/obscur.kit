The fallback UI to commit in place of the failed subtree — `Some`
exactly when `result` is [`RecoveryResult::FellBack`]. A retry leaves
the subtree to the next frame, and a propagated failure leaves it to
an enclosing boundary.
