The compose-only plan: only the scene is re-composed; reconcile, style and
layout are all skipped and reuse the previous frame's retained tree,
computed styles and boxes. Used for a same-size text edit
([`FrameReason::TextEdit`]) — the field re-shapes and repaints from the
live editing session while every box is provably unchanged.
