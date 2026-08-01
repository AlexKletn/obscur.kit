Installs `bridge`, replacing any previously installed one.

It does **not** reset the published-focus record, so the immediate push
the runtime makes after installing reports a `previous` that belongs to
the bridge before it — and where focus has not moved since, reports a
change from a node to itself, which
[`AccessibilityFocus::changed`] reads as no change at all. Preserved as
it was rather than quietly corrected: an adapter that acts on `focused`
(as `AccessKitBridge` does) is unaffected, one that gates on `changed`
would never learn the focus of the tree it was just handed, and deciding
between those two readings of the port's contract is a change of its own.
