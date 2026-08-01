Advances every live timeline by `delta` logical seconds, recording where
each one that finished on this tick settled, and reports whether any motion
remains — the caller's signal to keep asking for animation frames.

The settled progress matters because the scheduler *prunes* a finished
timeline while its binding lives on to hold the endpoint value.
