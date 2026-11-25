Drops `property`, returning what was there.

The inverse of [`set`](Self::set), and it exists because "there was no entry" and
"there was an entry holding the initial value" are different states: a caller
that overlays a value for one frame has to be able to restore either.
