The §5.15–§5.19 resource tier: the injected loader / decoder / vector ports,
the decoded-image and parsed-vector caches, the lifecycle hub, and the
channel finished off-thread loads return through.

One field rather than the fourteen it replaces, because those fourteen are
only ever correct together — see [`ResourceBridge`] for the invariant that
ties the cache, the handle map and the hub to each other.
