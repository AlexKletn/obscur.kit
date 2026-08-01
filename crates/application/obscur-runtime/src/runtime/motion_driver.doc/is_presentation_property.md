Whether `property` is a **group-compositing** property that motion applies as
a presentation value at paint time and never writes into computed style
(spec §4.17 post-layout motion): `opacity` (a `PushLayer` group alpha) and
`transform` (a `PushTransform` bracketing the subtree). Both are paint-only —
they move paint without ever re-running layout.
