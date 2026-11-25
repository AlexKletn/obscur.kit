Which pointer button a synthetic pointer event refers to.

This mirrors the small, fixed set of logical buttons a test needs to inject
for `PointerDown` / `PointerUp` events. It is inert typed data: the mapping
from a logical button to a concrete platform button is applied by the
interaction subsystem during dispatch (Phase 3), not here.
