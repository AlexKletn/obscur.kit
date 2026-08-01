End-to-end: a **drag-and-drop session** driven by real pointer input through
the live runtime, with its effect gated by a provisional transaction
(spec §3.2 commit/cancel, §6.21 drag and drop).

The audit's §3 headline was that `ProvisionalTransaction` was "implemented but
not connected to any interaction system" — and the drag domain
(`DragSession` / `DragSource` / `DropTarget`) was equally unconnected at the
other end. This is the interaction system between them.

What the commit/cancel gate buys is asserted rather than described: a drop
applies exactly the dropped-on node's work, and a drag that wanders across
targets and is then abandoned applies **nothing** — not one frame that
recomputes nothing, but no frame and no invalidation at all.
