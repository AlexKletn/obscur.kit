Borrows this session into the per-frame [`EditingView`] that BOTH the
layout (measure/arrange) and scene (compose) passes consume, so the focused
editable is sized and painted from the SAME live session state (CSS Text 3
§4.1.1): the measure pass shapes the session buffer to size the box, and
compose paints the identical string plus the caret/selection. The view
carries the field node, the session `text`, `selection`, `scroll_left`,
and — while composing — the live IME preedit (spec §5.12).

The runtime maps `None` (no session) straight through, which leaves both
passes byte-identical to the non-editing path.
