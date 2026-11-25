Returns `true` if `current` is newer than the observed generation, i.e.
the subscribed source changed since this subscription last synced.

`current` must be the generation of [`source`](Self::source); comparing
against a different source's generation is meaningless.
