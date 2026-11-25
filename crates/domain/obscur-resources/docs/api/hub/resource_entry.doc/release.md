Drops one reference and returns the new count (spec §5.15).

The count saturates at `0`; a return of `0` signals the resource is now
unreferenced and may be evicted by the hub.
