The handle onto this application's own running runtime: positional command
handlers, §2.3 listeners, stylesheet invalidation, and a queued
[`with_runtime`](AppControl::with_runtime) escape hatch.

Returns a clone of the one handle the runner applies, so a command handler may
capture it. Read [`AppControl`] for **when** a queued request takes effect — at
the next frame boundary, never inside the handler that asked — and for why
nothing hands out a `&mut UiRuntime`.
