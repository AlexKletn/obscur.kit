The content box: the border box inset by `border` then `padding`.

The content origin is offset by the top/left of `border` + `padding`, and
the content extent is the border-box extent minus the corresponding
edges. Each extent is clamped to be non-negative, so a box smaller than
its insets yields a zero-extent content rect rather than a negative one.
This never panics.
