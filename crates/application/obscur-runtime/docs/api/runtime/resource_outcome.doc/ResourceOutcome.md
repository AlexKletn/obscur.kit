What a finished resource-load task produced — the typed payload of the
[`TaskResult`](obscur_task::TaskResult) it delivers (spec §3.3 "results return
as typed messages").

Each variant carries the `handle` and `version` the job was started under, so
the hub can reject a completion whose resource has since been re-requested
(§3.3 "stale work is discarded using generation/version IDs"). The runtime does
not re-derive staleness: `ResourceHub::complete`/`fail` already own that check.

# Why the load lives here rather than on the bridge

Because the load runs where the bridge does not exist. A job is either handed
to a [`BlockingExecutor`](obscur_task::BlockingExecutor) and run on another
thread, or parked in a queue and run on the frame thread *after* the frame
phase (spec §3.3, §3.4) — and on neither path may it reach back into the
runtime. Everything it needs is therefore already in its hands when it starts:
the two ports, the candidate ladder, and the handle and version that identify
what it is answering for. That is what makes this an entity and not a method —
its whole interface is [`load_image`](Self::load_image) /
[`load_vector`](Self::load_vector) taking those arguments and returning this
value, while its internals (the §5.16 DPI ladder, the intrinsic size the
decoder reports, and the RGBA byte-length guard that catches a decoder
disagreeing with itself) are reachable from nothing else.

The seam is exactly the thread boundary, which is why it falls where it does:
everything above it — the resolve passes that decide *what* to ask for, and
the commit that rules on staleness and seats the pixels — reads the tree, the
styles and the caches, and so must stay on the frame thread with
[`ResourceBridge`](super::ResourceBridge). Everything below it is `Send` and
runs anywhere.
