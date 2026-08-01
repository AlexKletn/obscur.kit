Publishes an event for each of `runtime`'s transitions since the last
call. Cheap enough for every frame: two reads and two comparisons when
nothing changed.
