[`UiRuntime`]'s resource surface: the ports a host installs — loader, decoder,
blocking executor, vector decoder — the §5.19 image cache's budget and eviction
policy, and the counts a caller reads back.

Every one of these is a fact about the resource bridge, not about the frame
pipeline: the runtime holds the bridge, and this file is the API it states on the
runtime's behalf. The methods that do more than state it — the deferred-load
drain, the visibility marking and the three per-frame resolutions — stay with the
frame, because they read the tree, the boxes and the surface as well.
