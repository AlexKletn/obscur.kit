Returns the active IME preedit text, if a composition is live (spec §5.12).

This is *uncommitted* composition text shown inline (spliced for display,
underlined) but deliberately kept OUT of the buffer — so it is distinct
from [`editing_text`](Self::editing_text), which returns only committed
content. `None` when no composition is active.
