Parses an `An+B` micro-syntax argument into an [`style::AnB`].

Handles the keywords `odd` (`2n+1`) and `even` (`2n`), a bare integer `k`
(`0n+k`), and the general `An±B` grammar including the omitted-coefficient
forms `n` (`1n+0`) and `-n+B` (`-1n+B`). Whitespace inside the argument is
ignored. Anything unparseable degrades to `0n+0`, which matches no index.
