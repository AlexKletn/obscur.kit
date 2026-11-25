Removes every finished timeline, returning how many were dropped.

A timeline is dropped when [`Timeline::is_finished`] is `true`; those still
running, paused or idle are retained. This is how the engine prunes
completed animations from the active set (spec §4.17).
