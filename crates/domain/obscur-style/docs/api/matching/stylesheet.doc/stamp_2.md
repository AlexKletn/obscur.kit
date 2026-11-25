Whether two stylesheets carry the same rules in the same origin.

Deliberately blind to [`stamp`](Self::stamp): equality here is about content,
so a sheet compiled twice from the same CSS equals itself, and a caller asking
"are these the same rules?" gets the answer it means. A caller asking "is this
the same sheet I saw last frame?" wants the stamp instead — it answers in a
`u64` comparison what this answers by walking every selector of every rule.
