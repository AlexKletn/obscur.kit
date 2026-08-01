Scratch slot for the most recent ANONYMOUS parse.

An anonymous source is re-parsed every call (it has no identity to cache
by), but [`parse`](Self::parse) hands back a reference, so the result has to
live somewhere. It is deliberately a single overwritten slot rather than a
map entry: keying anonymous sources would make two unrelated sheets collide,
and the second would silently receive the first's rules.
