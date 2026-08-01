Post-edit bookkeeping: on a text-changing edit, dispatch the field's
`on-input` command with the session's resulting text; keep the caret in
view; then schedule the frame that flushes the edit to paint.

# Frame scheduling: compose-only vs full
When an `on-input` handler actually **ran** (a *controlled* field: the app
mirrored the value into its state), [`dispatch_text_input`] already
requested a full [`FrameReason::Input`] frame — the mirrored value must be
reconciled (and, for a normalizing handler, that reconcile is what drives
the Defect-A controlled-value reseed). So this does **not** schedule a
second, redundant frame.

When **no** handler ran (a caret move, or an *uncontrolled* field whose
session is the sole source of truth), no reconcile is needed, so a
box-stable edit is flushed with a COMPOSE-ONLY [`FrameReason::TextEdit`]
frame — the per-keystroke fast path (Defect B) that re-shapes + repaints the
field while skipping style + layout. An edit that can resize the box
(wrapping / multiline growth / auto width) still takes a full `Input` frame.
