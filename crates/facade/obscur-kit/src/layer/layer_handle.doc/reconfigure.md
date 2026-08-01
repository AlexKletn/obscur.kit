Asks for the surface to be reconfigured.

Applied at the top of the next loop turn, not immediately. The compositor
answers with a fresh configure, so the resulting size is whatever it
decides — read it back from [`size`](Self::size) rather than assuming the
request was honoured verbatim.

A second request before the first is applied replaces it: only the latest
intent matters.
