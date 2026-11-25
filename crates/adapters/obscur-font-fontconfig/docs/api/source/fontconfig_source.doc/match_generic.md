Resolves a CSS **generic** family to the face the host nominates for it.

This is `fc-match`, and substitution is the point: `sans-serif` is a
request for *whatever this host calls its sans*, so an answer that is not
literally named "sans-serif" is correct rather than a miss. Returns `None`
only when fontconfig is absent, answers nothing, or answers with a file
this toolkit cannot load (see [`is_loadable`]).
