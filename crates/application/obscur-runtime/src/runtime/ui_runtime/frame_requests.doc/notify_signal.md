Records that the value behind the observable `source` changed, and marks
every component that depends on it for re-render (spec §3.1).

This is the reactive entry point for application state. A host mutates its
own data — the toolkit provides no store and never reaches into it — and
then names what changed. The runtime invalidates exactly the components
whose last render read that source: one that read a different source, or
none, is untouched. That is what makes a state change cost its consumers
rather than the document.

A frame is requested only when something actually depends on the source,
so notifying a source nobody reads is free. Returns how many components
were invalidated.
