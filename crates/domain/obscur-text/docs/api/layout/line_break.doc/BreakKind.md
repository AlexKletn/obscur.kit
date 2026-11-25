The kind of a [`LineBreak`](super::LineBreak) opportunity (spec §5.10).

A [`Soft`](Self::Soft) break is a *permitted* wrap point (for example after a
run of whitespace) that a line-breaking pass may or may not take; a
[`Hard`](Self::Hard) break is *mandatory* (for example a `'\n'`) and always
ends the line.
