Services queued backend operations before the loop blocks for input.

Property/focus/close requests (and any late `create_window` calls) are
enqueued by the backend between loop turns; re-pumping here applies them
promptly without waiting for the next OS event.
