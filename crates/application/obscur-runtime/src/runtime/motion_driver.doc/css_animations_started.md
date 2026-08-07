Which `@keyframes` animations have already started, so a rule that keeps
matching does not restart every frame.

Swept by [`retain_live`](Self::retain_live) when a node leaves the tree —
which is also what lets a *remounted* node start its animation again, since
the entry recording the old mount's start is gone with it.
