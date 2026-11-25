Something outside the loop asked it to reconsider the interface.

Carries no reason, because [`LoopWaker`](crate::port::LoopWaker) has none to
give. A thread with news rings the bell; the loop turns and rebuilds the view
from whatever the shared state now says.
