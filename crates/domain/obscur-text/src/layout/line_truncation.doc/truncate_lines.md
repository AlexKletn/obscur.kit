Truncates each line whose content width *still* exceeds `max_width` — a single
unbreakable, un-hyphenatable word is the usual cause — under the `overflow`
policy (spec §5.10).

A line at or under `max_width` is untouched, so this is a no-op for a
paragraph that wrapped cleanly. The caller decides whether the pass applies at
all (a truncating policy, a positive finite box); every line it does reach is
cut by [`truncate_line`].
