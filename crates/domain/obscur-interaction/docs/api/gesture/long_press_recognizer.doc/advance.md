Adds `seconds` to the time the contact has been held, returning `true` on
the single step that crosses the threshold.

Non-finite time steps are ignored.

Returns `false` once terminal, so the caller may keep ticking every armed
recognizer without filtering.
