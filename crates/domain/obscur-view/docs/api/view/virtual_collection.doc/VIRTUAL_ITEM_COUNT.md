Reserved symbol for a virtual list's **item count** — how many items exist,
not how many are built (spec §4.9).

The marker stores the count as a nonnegative integer or, beyond the signed
integer range, as decimal text. Both forms decode to `usize`.

This is the half an app-side virtualization cannot express. Because the runtime
knows it, it can give the scroll container the true content extent
(`count * row_extent`) while realizing only the window — so the scrollbar is
honest, the wheel is native, and nothing about paging is the app's arithmetic.
