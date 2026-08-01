Whether a soft `break` is suppressed by `word-break: keep-all` because it
sits between two adjacent CJK ideographs (CSS Text 3 §5.2).

A mandatory (forced) break is never suppressed. A soft break at byte offset
`o` is suppressed when the character ending at `o` and the character
starting at `o` are both CJK ideographs — i.e. it is one of the implicit
inter-ideograph opportunities `keep-all` forbids.
