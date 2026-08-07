Change-detection baseline: the settled value each watched property last
resolved to, per node.

Keyed by `(NodeId, PropertyId)` and cleared wholesale only when the stylesheet
set is replaced, which for a running program may be never — so
[`retain_live`](Self::retain_live) drops the entries whose node has left the
tree. A `NodeId` is generational, so a dead entry is never reclaimed by a
remount and this is the only thing that ends it.
