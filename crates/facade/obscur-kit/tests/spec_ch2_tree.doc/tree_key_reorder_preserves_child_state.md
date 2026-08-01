§2.2: "child identity is stable through explicit keys;" and "layout, focus,
selection, animation, DnD and caches attach to retained identity."

FIXTURE REPAIRED 2026-07-19: `rows()` used to build each row as
`keyed(100 + i, "row", key, [el(200 + i, …)])`, making the row's
ComponentId — its element *type* — a function of its POSITION in the list.
Reordering therefore changed each row's type, which is the one thing that
legitimately defeats key-based identity, so the fixture contradicted the very
premise it asserts. The ComponentId is now constant across rows and identity
is carried by the key alone.

OBSERVATION MOVED 2026-07-19: with the fixture repaired, every identity
assertion passes — row b's NodeId is unchanged, it lands third, and the tree
gains and loses no nodes. What did not survive is
`rt.editing_text() == Some("hello")`, and that assertion was reading the
wrong thing: `editing_text` is a RUNTIME-GLOBAL `TextEditSession` whose text
is reseeded from the declarative tree on commit (and this row's input
declares no value), so it measures value flow-down (§3.1), not state attached
to retained identity. It is replaced by the part that does test identity —
the session survives the reorder at all, which it could not if row b's node
had been destroyed and recreated. Candidate spec amendment: §2.2 lists what
"attach[es] to retained identity" (layout, focus, selection, animation, DnD,
caches) but not text-editing state; say whether an editing session is
per-node retained state, and how it interacts with §3.1's downward value flow.
