Present into an image the caller already holds, running `before_commit` in the
instant between the last thing that can fail and the commit itself.

# Why the hook exists

A `wp_presentation` feedback and a `wl_surface.frame` callback both bind to the
surface`s **next** commit, so both have to be requested before the commit and
after everything that might mean there is no commit. There is exactly one such
instant, and it is inside this function: the commit is `SurfaceTexture::present`,
and the render, the blitter and the view creation above it can all return early.

A caller that asks outside this function is making a bet. When it asks before
calling, a failed render means the request binds to whatever the process commits
next — a scale sync, a settings apply, the following frame — and from then on
every answer is about a different frame than the one the caller thinks it is
about, with nothing in the log to say so. When it asks after, it has bound the
request to the frame after the one it meant to measure.

Everything else is [`present_to_texture`], which passes a hook that does nothing.
