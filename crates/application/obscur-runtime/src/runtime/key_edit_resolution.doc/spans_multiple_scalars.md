Whether `cluster` is made of more than one Unicode scalar — a ZWJ emoji
sequence or a base+combining-mark cluster, as opposed to the single-scalar
cluster that is the common case (spec §5.11 / §5.10).

A single-scalar cluster is left to the buffer's own one-`char` deletion (and
its coalescing); only a multi-scalar one has to be selected and deleted as one
edit, which is what [`grapheme_cluster_range`](KeyEditResolver::grapheme_cluster_range)
uses this to decide.
