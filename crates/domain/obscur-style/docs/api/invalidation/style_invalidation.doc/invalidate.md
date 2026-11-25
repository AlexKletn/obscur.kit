Records that `node` must be restyled for `reason`.

If `node` was already invalidated the existing reason is kept (earliest
reason wins), so the record is deterministic regardless of call order.
