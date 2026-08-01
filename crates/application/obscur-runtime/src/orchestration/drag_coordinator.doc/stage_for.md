The work a drop on `node` would cause: that node re-renders, because it is
about to receive the payload and its content changes.

Deliberately narrow. The drag's *source* is not staged: whether the item
leaves its origin is the application's decision (a `Copy` leaves it in
place), and the runtime does not get to assume it.
