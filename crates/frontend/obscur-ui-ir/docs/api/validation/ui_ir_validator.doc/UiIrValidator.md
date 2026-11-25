A stateless validator for [`UiIrDocument`] trees (spec §2.5).

[`validate`](UiIrValidator::validate) walks the whole tree once, threading a
[`ValidationContext`] for cross-node bookkeeping, and returns a
[`UiIrValidationResult`] collecting every fault found. The checks are purely
structural well-formedness rules — they do not require a host or any I/O:

- **duplicate ids** — every node's stable id must be unique across the tree;
- **empty tags** — `Element`/`Component` nodes must name a tag/component;
- **text leaves** — `Text` nodes must not have children;
- **empty bindings** — a property bound to `Binding("")` has no path.

In addition the document's `required_capabilities` are checked for
well-formedness: any capability whose id string is empty is reported as an
[`UndeclaredCapability("")`](UiIrValidationError::UndeclaredCapability).
Framing note: the deeper "does every binding-referenced capability appear in
`required_capabilities`?" analysis is a compiler concern and is intentionally
out of scope here; this validator only checks that the declared set is
itself well-formed (non-empty ids).
