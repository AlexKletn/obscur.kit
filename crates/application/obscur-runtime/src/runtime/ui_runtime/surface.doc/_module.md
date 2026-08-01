[`UiRuntime`]'s surface surface: the viewport the document lays out into, the
device scale and content offset that map it to the host's pixels, the viewport
scroll `position: sticky` clamps against, and the §6.16 environment values a
platform reports — safe-area and keyboard insets.

The insets are here because they are geometry a host measures and hands in, even
though they travel as `env()` values through the cascade: the pair of setters
does nothing but fold one inset set into the environment, and the environment is
what the viewport means once a platform has taken a bite out of it.

`set_environment` is the one method in the file that reaches past the geometry,
and only in one direction: a keyboard that has just risen is the reason §6.15
says "focused inputs may be scrolled into view", so a change to the keyboard
inset — and nothing else in the environment — reveals the focused editable. That
call belongs to the *change*, not to scrolling: nothing about the focus moved,
so only the layer that saw the inset move can know a reveal is owed.

`invalidate_dpi_variants` is private and has exactly two callers, both above it:
a surface or a scale change can select *different image files* off the §5.16
DPI-variant ladder, and the resident decodes were chosen against the old one.
