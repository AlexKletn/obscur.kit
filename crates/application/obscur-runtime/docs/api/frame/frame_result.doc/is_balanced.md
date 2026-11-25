Whether the published snapshot's display list is well-formed (balanced).

Delegates to [`SceneSnapshot::is_balanced`]; a balanced list has every
push paired with a matching pop and is safe for a renderer to replay.
