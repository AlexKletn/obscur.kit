Replacing a virtual list's content returns it to the top.

A scroll offset belongs to the content it was scrolled through. When an app swaps
a list's items — a file manager entering another directory — the runtime cannot
see that the model changed: the marker looks the same and the container is
reconciled in place, so its scroll survives and the new list opens scrolled into
nowhere. The app states a scroll-reset token on the marker; a change in it resets
the scroll. These tests pin that a changed token returns the window to the top and
an unchanged one leaves the scroll where the user left it.
