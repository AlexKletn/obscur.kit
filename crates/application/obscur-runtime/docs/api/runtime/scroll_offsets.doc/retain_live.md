Drops the offset of every node the tree no longer holds.

`offsets` is populated by every node whose offset was ever *written* — a superset of "ever scrolled", since the `VirtualFor` model reset and the caret-follow both seat one — and nothing removed an entry, so the map counted mount/unmount generations rather than live containers. A dead entry is unreachable rather than wrong (`refold` builds from the live containers and a generational id never matches a remount), so this is a bound, not a repair.
