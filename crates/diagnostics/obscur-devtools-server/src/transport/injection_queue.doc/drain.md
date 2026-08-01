Takes every pending event, leaving the queue empty.

The host calls this from the thread that owns its runtime (typically once
per frame) and applies each event.
