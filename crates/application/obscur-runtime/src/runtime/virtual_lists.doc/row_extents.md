Each virtual list's estimated row extent, captured when its window is realized.
This is what makes a click on a row resolvable to a model index: the extent
maps a y-offset within the container to a row. Only the runtime holds it — the
same reason it, not the app, owns the window and the scrollbar's extent. A
bound against the item count is deliberately not kept: a tracked container has
spacers, so its content is at least a viewport tall, so every point inside its
(clamped) scrollport already lands on a real row.
