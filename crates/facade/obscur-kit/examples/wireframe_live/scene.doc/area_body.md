03 · Textarea: a live **multi-line** [`Input`] (`.multiline(true)`), which
emits the reserved CSS type `textarea` so the UA `textarea { … }` rules bind
(`white-space: pre-wrap` — it wraps at the box AND preserves `\n`). Enter inserts
a newline and ArrowUp/ArrowDown move between wrapped lines (runtime MULTILINE).
