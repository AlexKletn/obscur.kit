Computes specificity for parsed selectors and mints their [`SelectorId`]s.

Ids are handed out sequentially (starting at zero) across the lifetime of the
compiler, so every selector it compiles gets a distinct handle.
