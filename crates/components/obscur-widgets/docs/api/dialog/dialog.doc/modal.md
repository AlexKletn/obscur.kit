Sets the modal flag, returning the updated dialog (fluent builder).

A modal dialog traps focus: while it is open, Tab and Shift+Tab cycle within it,
a pointer press outside it moves no focus, and closing it returns focus to
whatever had it when the dialog opened. `modal(false)` asks for none of that —
the dialog is drawn, and focus treats it as ordinary content.
