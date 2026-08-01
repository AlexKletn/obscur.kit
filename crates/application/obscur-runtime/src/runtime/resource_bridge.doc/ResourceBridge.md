Everything the runtime's resource tier owns: the injected ports, the caches
the decoded assets live in, the hub that rules on each resource's lifecycle,
and the channel finished off-thread loads return through.

# Why this is one entity rather than fourteen fields on the runtime

These fourteen pieces of state are only ever meaningful together. A handle in
[`image_handles`](Self::image_handles) is the "already requested" guard for a
URL whose pixels are in [`image_cache`](Self::image_cache) and whose lifecycle
is in [`resource_hub`](Self::resource_hub); evicting from the second without
releasing the first is the bug [`release_evicted`](Self::release_evicted)
exists to prevent, and it is a bug the type system can only help with while
the three sit behind one owner. Held as loose fields on `UiRuntime` they were
reachable from all 254 of its methods; held here, the frame path reaches them
through the dozen operations below and nothing else can.

# What it deliberately does not hold

It has no view of the tree, the styles, the viewport or the diagnostics
channel. Each operation that needs one takes it as an argument — see
[`ResourceReporting`] for the fault-reporting half — so the dependency is
visible at the call site instead of implicit in a shared `self`.
