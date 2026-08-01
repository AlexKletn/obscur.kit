The retained layout results `dirty` can have changed: which nodes must
re-`measure`, and which must re-`arrange`.

Both sets come out of **one** dependency analysis and one tree walk, because
they are two readings of the same fact — see [`invalidated_layout`].
