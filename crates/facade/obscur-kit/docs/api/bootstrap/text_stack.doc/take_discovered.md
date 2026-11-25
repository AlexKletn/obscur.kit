Takes the faces discovered since the last call, as `(handle, bytes)`.

The startup table above is not the end of the story: a character outside
every registered face pulls one in from the host mid-frame, and the
renderer rasterizes from its own id→bytes table — it would draw nothing
for a face it has never been given. A host calls this each frame and
registers what it finds, which is the same hand-off as at startup,
continued.

Empty on every frame where nothing new appeared, which is almost all of
them.
