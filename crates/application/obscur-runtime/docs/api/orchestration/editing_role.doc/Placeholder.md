`node` is inside the field's `.placeholder` overlay (the CSS `::placeholder`
pseudo-element). By the `:placeholder-shown` model the placeholder is shown
*iff the field's effective displayed text is empty*: while editing, the
effective text is the SESSION buffer, not the stale value. So the session
overrides the placeholder slot **only while the session has typed text**
(`!text.is_empty()`); an empty session leaves the muted hint painted.
