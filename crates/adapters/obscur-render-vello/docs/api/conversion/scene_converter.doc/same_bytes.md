Are these two face byte-slices the same face?

The answer that matters is the second clause: a byte comparison, which is what
this always was. The first clause is only a shortcut for the case the kit now
makes common — the host handing back the same `Arc<[u8]>` it handed over before
— where address and length settle it without walking 26 MB. Two slices at the
same address with the same length ARE the same bytes, so the shortcut cannot
disagree with the comparison it skips; it can only skip it.

Why either is needed at all: replacing a face with itself would silently throw
away every glyph outline Vello has cached for it, because a `Blob`'s identity is
a fresh global counter value and that identity is the cache key. `register_font`
explains the rest.
