Called when the desktop's appearance has changed.

Carries **nothing** — not which key, not the new value. A caller reads the whole
appearance again, which is one cheap call, and in exchange nothing here has to
parse a second format to describe a change it is about to be asked about anyway.

`Send + Sync`, because it is called from wherever the source's notification
arrives, which is not the thread that asked to be told.
