The resource work a **frame** does, as opposed to the ports and cache settings a
host installs — those are `resources.rs`'s.

Two things live here and they meet in the middle. The first is *landing*:
[`poll_resource_tasks`](UiRuntime::poll_resource_tasks) and
`commit_resource_results` are the §1.5 synchronization point at which a load that
finished elsewhere becomes state this runtime can render, one of them asking for
a frame afterwards and the other deliberately not — because the frame path calls
it from inside a frame it is already running. The second is *resolving*: the
three per-frame passes that turn every `background-image` URL and every element
`src` this frame references into a cached decode, plus the visibility marking
that runs first so an eviction drops what the last frame stopped showing rather
than what this one is about to paint (§5.15–§5.17, §5.19).

They are the frame's rather than the bridge's because each reads something the
bridge deliberately does not hold: the retained tree, the arranged boxes, the
surface's device scale, and the diagnostics sink every failure is reported
through. That is the same rule that put them here and left the port installation
next door.
