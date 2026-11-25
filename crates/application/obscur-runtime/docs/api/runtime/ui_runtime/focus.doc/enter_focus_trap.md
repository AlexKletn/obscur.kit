Takes focus into a trap that has just opened, remembering what it displaced.

The frame is saved first and unconditionally — including when nothing was
focused, which is the case a `Vec<NodeId>` stack could not record and the reason
the frames are optional. Focus already inside the trap is left where it is: a
dialog that renders with its own field focused must not be dragged back to its
first control by the frame that merely noticed it.

A trap with no tab stop at all moves nothing. There is nowhere to go, and
focusing the dialog's frame — a node no keystroke can use — would only make the
next Tab start from a stop the user cannot see.
