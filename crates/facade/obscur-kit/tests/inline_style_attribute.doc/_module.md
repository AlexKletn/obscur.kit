The `style` attribute, end to end: authored on a node, compiled by the CSS
front end the facade installs, folded into the cascade at origin `Inline`
(§4.14), and read back out of a laid-out frame.

Everything below the facade has its own unit tests — the block type, the
priority arithmetic, the compile cache, the attribute extraction. None of them
can see the thing that was actually missing for as long as this feature existed:
that *nobody installed the compiler*. Only a test that boots the way an
application boots can, which is why this one lives here and drives a real frame
rather than a matcher.
