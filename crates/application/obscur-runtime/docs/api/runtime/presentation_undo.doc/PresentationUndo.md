What a presentation overlay wrote, and what was there before it.

Motion's sampled `transform` and `opacity` must reach paint and must never be
observable through `computed_style` (spec §4.17). That used to be arranged by
composing against a deep copy of every node's style; this is the record that
makes it arrangeable by writing the samples into the real map and taking them
out again, which is work proportional to the overlays rather than to the tree.

Each entry carries the node, the property, whatever value the property held
before (`None` if it held none), and whether the *node* had no style at all —
because restoring a node that motion itself introduced means removing it, not
writing an empty style back.
