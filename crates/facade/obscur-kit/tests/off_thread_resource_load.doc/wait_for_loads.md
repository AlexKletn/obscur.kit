Waits until the recording loader has seen `n` calls, or the deadline passes.

A bounded poll rather than a sleep: the pool decides when the job runs, and
the test must not assert on timing. Failing to reach `n` is left to the
caller's assertion, which reports what was actually seen.
