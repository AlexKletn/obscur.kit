The result of compiling one `style` attribute.

Two halves that must not be separated. The declarations are what the cascade
folds in at origin `Inline`; the faults are what the author has to be told
about, because a `style="width: nope"` that silently declares nothing looks
exactly like an element with no attribute at all. Spec §1.6 asks for a
localized failure *and* a report, and a compiler signature that returned only
the block could deliver the first but never the second.
