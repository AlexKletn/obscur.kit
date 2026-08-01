Every registered `<VirtualFor>` list (spec §4.9): its row builder, its row
click and long-press handlers, and the container, window and row extent it
realized against last frame.

One field rather than the seven it replaces, because a window, the extent it
was realized at and the container it landed in are one fact — see
[`VirtualLists`], which is why no window can be recorded without the extent
that makes its rows clickable, and why a replaced model's scroll reset
cannot land after the window that reads it.
