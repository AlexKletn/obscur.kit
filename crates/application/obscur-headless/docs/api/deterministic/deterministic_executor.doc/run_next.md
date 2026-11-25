Runs exactly one queued job (the front of the queue).

Returns `true` if a job was run, or `false` if the queue was empty. The
job is popped while the lock is held, the guard is dropped, and only
then is the job invoked — so a job that `spawn`s during its run does not
deadlock on the queue.
