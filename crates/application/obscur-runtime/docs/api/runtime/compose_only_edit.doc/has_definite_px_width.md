Whether `style` carries a **definite** absolute-`px` `width` — a concrete
length, not `auto`/content-based sizing.

A definite-width field cannot grow or shrink with its typed content, so its
own box (and every ancestor's) is invariant across a text edit — the
precondition for a compose-only frame to reuse the retained boxes unchanged. A
percentage or `auto` width returns `false` (conservatively taking the full
frame), mirroring the scene coordinator's pixel-only length reads.
