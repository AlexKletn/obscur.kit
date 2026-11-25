Ends a sequence the system took away — a gesture claimed it, the window lost
the contact, the device went away — and returns whether the `:active` styling
state changed.

The press is discarded **without** a click: an interrupted gesture must not
activate the control the pointer was resting on, which is the one thing
separating a cancel from a release. A cancelled pointer has completed just as
surely as a released one, so its hold, pan, capture and drag-selection end the
same way; nothing it was carrying is thrown.
