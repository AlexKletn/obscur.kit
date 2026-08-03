The bracket kinds currently open, innermost last.

The builder used to keep only a COUNT. That was enough to say whether a list
was balanced and not enough to close it correctly: nothing could tell whether
the innermost open bracket was a transform, a clip or a layer, so an unwinder
had to guess. It also means a `pop_clip` closing a `push_transform` still
counts as balanced — a structurally wrong list the renderers would obey.
Recording the kind is what makes [`pop_any`](DisplayListBuilder::pop_any)
possible, and it is what a future well-formedness check would read.
