The solid brush the display list painted `text` with, if it painted it.

Read off the scene rather than off the computed styles because a synthetic
part has no node to compute a style for: the §4.16 placeholder is resolved
straight into a paint command, so the paint is the only place the answer
exists.
