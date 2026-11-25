Pops the most recently saved frame and answers what it named.

`None` means the closing scope displaced nothing worth returning to — either it
opened while nothing was focused, or (a caller's bug) there was no frame to pop
at all. [`depth`](FocusRestore::depth) is what distinguishes the two; a caller
that pushes exactly one frame per scope it opens never needs to.
