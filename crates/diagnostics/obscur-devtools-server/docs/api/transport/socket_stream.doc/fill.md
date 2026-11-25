Fills `buf` from `filled` onward, advancing `filled` by whatever arrives.

Returns `Ok(true)` once `buf` is full, `Ok(false)` on a clean end of stream —
which is only clean when `at_start`, i.e. nothing of this frame had arrived
yet; an EOF mid-frame is a truncated message, not a polite disconnect.

**`filled` is the caller's state, not a local.** On a would-block/timeout the
error propagates with `filled` already advanced past the bytes that did land,
so the next call resumes rather than re-reading them as a fresh frame header.
