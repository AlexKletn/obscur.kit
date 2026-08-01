Whether the builder is currently balanced, i.e. [`depth`] is zero.

Note this tracks the running depth clamped at zero; a builder with a
stray pop followed by a balancing push reports balanced here, yet the
resulting list is not — use [`DisplayList::is_balanced`] on the built
list for the authoritative check.

[`depth`]: DisplayListBuilder::depth
