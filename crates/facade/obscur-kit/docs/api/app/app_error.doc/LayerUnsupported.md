This desktop has no layer surfaces, so no panel can be opened.

Expected on GNOME, whose compositor has declined the protocol, and on any
platform for which no desktop-surface backend exists. A capability answer to
branch on: the fallback is an ordinary window, reached by not calling
[`App::layer`](crate::App::layer).
