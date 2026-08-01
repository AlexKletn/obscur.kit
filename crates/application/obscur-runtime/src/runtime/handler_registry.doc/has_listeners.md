Whether any listener is registered anywhere.

The propagation walk asks this first: with nothing registered there is no
one to run, and building a node path to hand to nobody would cost two
allocations per click. That is what makes §2.3's phases free for an
application that never asks for them — the walk is skipped, not merely
empty.
