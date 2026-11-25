A hand-written recursive-descent parser over the pattern bytes.

Byte-oriented: every token it looks for (`{`, `}`, `,`, `#`) is ASCII, and it
only ever slices at those positions, so multi-byte text passes through intact.
