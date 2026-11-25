The author sheets one frame matches, as
[`author_for_frame`](CascadeSheets::author_for_frame) resolved them.

A token rather than a container: it carries no accessor, because the only
thing anything does with the author half is hand it back to
[`cascade`](CascadeSheets::cascade). That is the point. `cascade` accepts
nothing else and `author_for_frame` is the only thing that mints one, so the
two halves of "which sheets does this frame cascade" can only be performed
once, in that order — see [`CascadeSheets`] for what goes wrong when the
cascade list is mistaken for the author list.
