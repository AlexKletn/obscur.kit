Monotonic change token stamped onto a value each time it is mutated.

A consumer records the `StateGeneration` it last observed and later compares
it against the source's current generation to decide whether it must react.
This is the mechanism behind the spec's "invalidate only consumers of
changed values": equal generations mean nothing changed, so no work is done.

Backed by [`VersionStamp`] so the whole workspace shares one change-versioning
primitive rather than re-inventing a raw counter. Generations are totally
ordered; a larger value is strictly newer.
