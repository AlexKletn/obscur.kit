The frame-lifecycle contract, observed: a handler queues the invalidation, nothing
happens while it runs, and the next frame carries it. A handler cannot install a
stylesheet itself — it is called from inside §2.3's propagation, with the runtime
mid-frame — so the queue is what makes this expressible at all.
