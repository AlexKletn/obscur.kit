The font handle of every non-empty [`DrawText`](PaintCommand::DrawText) run in
`list`, deduplicated, in first-use paint order.

Both in-tree backends report [`SceneEffect::Text`] as
[`SupportLevel::Limited`] for the same reason — a run whose font the host never
registered does not draw against that font — but each resolves a handle by its
own rule (the CPU backend demands an exact, parseable match; Vello falls back to
the primary id `0`), so each maps these handles through its own registry to
decide which runs actually degraded. Reading the *list* is common, and lives
here so a backend does not re-walk it.

Empty runs are omitted: a run with no glyphs draws nothing whether or not its
font resolves, so it is not degraded by a missing one.
