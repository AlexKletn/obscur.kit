Drops every entry whose node the tree no longer holds.

The cascade inputs are retained across frames so an unchanged node need not be re-matched. Nothing in that model says a *departed* node must be kept, and the invalidation path cannot reach one: `RestyleScope::expand` returns early for a root absent from the tree, so the dirty set is always a subset of the live tree and a departed entry is never visited again — only paid for, in memory and in every pass that walks the map whole.

Called on a reconciled frame, which is the only frame that can remove a node.
