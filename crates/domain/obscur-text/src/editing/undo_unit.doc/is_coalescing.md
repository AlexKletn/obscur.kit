Returns whether two adjacent units of *this same* kind may merge into one
undo step. Only [`Typing`](Self::Typing) and [`Deletion`](Self::Deletion)
coalesce; the atomic kinds never do.
