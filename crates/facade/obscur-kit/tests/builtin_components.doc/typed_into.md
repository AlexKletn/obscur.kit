Types `a`, `Enter`, `b` into the mounted field of class `class` and returns
the editing session's text.

The pointer press is not incidental: the runtime seeds an editing session
from the focused editable's descendant `Text` content, so this exercises the
component's output as well as the host's local name. `Enter` is the question —
a multiline field takes it as a newline, a single-line one as an activation
that inserts nothing.
