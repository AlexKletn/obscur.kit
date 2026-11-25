A pointer sequence was taken away — `on:pointercancel="…"`. A system gesture
claimed it, the window lost the contact, or the device went away.

`prevent_default` **has nothing to suppress here, deliberately.** What follows a
cancel is not a default action but the unwinding of state the system has already
destroyed: the press is discarded without a click, capture is released and the
pointer is completed. A runtime that honoured a refusal would keep a `:active`
that never clears and a capture that never releases, for a contact that no longer
exists — the exact defect `cancel_landed` was written to fix. The event reports
the loss; it does not offer to decline it.

It is dispatched before that unwinding runs, so a handler still sees the press
and the capture it is about to lose.
