Reusing one key across siblings must be *reported*, not merely slow.

Keyed identity assumes a key names one child (§2.4). When several claim it the
matcher keeps the first and the rest find no partner: they are inserted as new
while their predecessors are removed, every frame. The UI still looks right — it
is only an order of magnitude slower — so without a diagnostic an author has no
way to connect the symptom to the cause. This was found the hard way: a probe
blanked the string a `<For>` used as its key, and the resulting churn read as a
layout pathology in the engine for several rounds of investigation.
