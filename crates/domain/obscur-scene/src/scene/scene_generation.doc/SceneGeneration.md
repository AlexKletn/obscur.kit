A monotonic generation counter stamping the content of a
[`Scene`](crate::scene::Scene) (spec §1.6).

Each content change advances the generation via [`SceneGeneration::next`],
so a published [`SceneSnapshot`](crate::scene::SceneSnapshot) can be checked
for staleness against the live scene: a **newer** generation (a greater
value) means the scene changed after the snapshot was taken, and the
snapshot may be discarded as stale.

This is a newtype over [`VersionStamp`]; ordering follows the underlying
stamp, so `a < b` means generation `a` is older than `b`.
