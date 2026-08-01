Returns the logical parent **only when it is an element node**. When the
parent is the synthetic root (or the node is detached), returns `None`,
so the node reads as top-level and `:root`/child combinators behave.
