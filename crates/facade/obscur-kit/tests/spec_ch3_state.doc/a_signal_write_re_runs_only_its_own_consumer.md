§3.1 — "Dependency tracking invalidates only consumers of changed values."

Writing signal A re-runs X's view function exactly once and Y's not at all;
Y's retained nodes keep their identity.
