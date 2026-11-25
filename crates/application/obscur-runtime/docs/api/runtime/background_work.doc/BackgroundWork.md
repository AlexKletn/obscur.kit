Pending background units and their lifetime service count (§3.6).

Runnable units share the scheduler's budget with accounting-only requests.
The runtime services runnable units in enqueue order, then accounts for any
remaining units without callbacks.

The first request and the final service report transitions to the runtime,
which updates the frame scheduler's fairness state.
