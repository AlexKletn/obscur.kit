Whether this is a **multi-line** field (a `<textarea>`, spec §2.6). A
multi-line field emits the reserved CSS type `textarea` (so the UA
`textarea { … }` rules bind and it wraps + preserves segment breaks, CSS
Text 3 §4.1.1) and marks its editable MULTILINE for the runtime, so `Enter`
inserts a newline and `ArrowUp`/`ArrowDown` move between laid-out lines. The
default (`false`) is a single-line `input` — byte-identical to before.
