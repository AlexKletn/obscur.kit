Transforms a desired declarative tree into the retained tree, preserving node
identity across moves via keys.

Reconciliation is exactly the desired → retained transform (spec §2.2):
updates are expressed as new desired state, and this engine is the sole
writer that turns that state into retained-tree structure. It never invents a
competing identity — it reuses an existing [`NodeId`] whenever a desired child
is compatible with an existing one (matching by key across reorders, per spec
§2.11), and mints a fresh id only when it must.

**Structure**, **identity**, **text content**, an element's **full authored
property list** (plus the CSS-relevant [`StyleAttributes`] derived from it),
and its **event bindings** (`event → command`) are all reconciled here.

Properties are retained rather than projected away because values flow *down*
(spec §3.1) and the desired tree is gone by the time downstream phases run: a
component's typed props are decoded from the retained list after reconcile,
exactly as a fired event finds its command in the retained bindings. The
change check compares the whole list, so a change to *any* authored prop — not
just a CSS-relevant one — yields an `Update`.

**A §2.12 slot outlet is refused**, never materialized: `project_slots` has
already substituted away every outlet in a component's output, so one reaching
this engine belongs to a view no projection will run over and could only become
an empty box nothing can fill. That refusal is what makes "an outlet never
becomes a retained node" true of the tree rather than of one code path; it is
silent here, because this engine has no diagnostics channel —
`UiRuntime::report_stray_outlets` is what tells the author.

Every pass returns a [`ReconciliationPlan`] describing what it did. The engine
is deterministic and panic-free, and is defensive about stale ids.
