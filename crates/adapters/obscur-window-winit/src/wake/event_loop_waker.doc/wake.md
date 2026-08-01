Wakes the event loop to service a scheduled frame (spec §3.5).

A send failure means the loop has already exited; there is nothing left to
wake, so the error is deliberately ignored.
