§6.15 — the host reports its insets, CSS `env()` lays out around them.

End-to-end through the real stack: authored CSS → the CSS frontend →
computed style → layout. The interesting part is not that `env()` parses; it
is that the value comes from the *host*, changes while the app runs, and that
a change actually reaches the arranged geometry rather than being memoized
away.
