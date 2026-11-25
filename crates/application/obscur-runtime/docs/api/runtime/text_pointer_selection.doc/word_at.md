The word containing byte `offset` in the focused editable (double-click,
spec §5.11), bracketed with the buffer's `NavigationUnit::Word` engine.

The word is bracketed by a forward word move from the click (to the word's
end) then a backward word extend (to its start), so the click may land
anywhere within — or at the leading edge of — the word. The buffer's word
motion is a simplified UAX-29 approximation (whitespace-delimited runs).
The returned `[start, end)` is **normalised to a forward selection** (anchor
at the word start) for the caller to set through its non-mutating edit path.
