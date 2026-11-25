Resolves the live [`ScrollPaint`] (clamped `scroll_top` + `content_height`)
of every overflowing scroll container, keyed by [`NodeId`], for the scene
coordinator to paint the scroll offset + scrollbar. Empty when nothing
overflows → a byte-identical compose.

`computed` is taken separately because compose resolves this against the
styles the frame just produced, which are not yet the retained ones.
