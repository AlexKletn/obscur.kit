Commits the live composition (spec §5.12 "commit"): marks it committed,
clears it, and returns the buffer byte range the caller must replace with
the committed text (the anchor span). Returns `None` when no composition is
active; the buffer replacement itself (one undo unit) is the caller's job.
