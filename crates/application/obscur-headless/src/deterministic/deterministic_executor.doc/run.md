Runs queued jobs in FIFO order until the queue is empty.

A job may itself [`spawn`](Self::spawn) more jobs; those run later within
the same drain, after all jobs already ahead of them, preserving order.
Each job is popped out of the lock before being invoked, so spawning
during a run never contends with the guard.
