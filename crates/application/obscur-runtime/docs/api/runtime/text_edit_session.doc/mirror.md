The **last retained value** the session reconciled against — the watermark
the controlled-input reseed compares the tree against (WHATWG HTML
§4.10.5.5 *value reflection*). Seeded to the field's text at focus and
advanced every structural frame the retained value the session sees
changes. It lets the runtime distinguish an EXTERNAL rewrite of the field
(a "Clear" button, or a normalizing `on-input` that writes back a
different value) — which must reseed the buffer so the field shows the
model value — from a field whose retained value simply never changes (a
static *uncontrolled* input), which must NEVER be reseeded lest it fight
the user's in-flight edit. Distinct from [`committed`](Self::committed)
(the `change` baseline) and from the live [`buffer`](Self::buffer) text.
