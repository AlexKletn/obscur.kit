One theme tab; the active tab gets the `active` class and an underline child.
The tab binds `on-click → cmd` ([`EVENT_CLICK`]) so a host runtime can resolve a
click on it to that command (the classes/CSS are unchanged; an unhandled command
is a no-op, so the static PNG examples keep rendering identically).
