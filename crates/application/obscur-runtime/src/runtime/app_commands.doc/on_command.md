Registers `handler` as the application fallback for `command`, replacing
any handler previously registered for that symbol.

The handler runs only when no window-level handler, and nothing in the
focused node's chain, claimed the command first (spec §2.6).

`handler` may take nothing or take the propagating [`UiEvent`] — see
[`IntoCommandHandler`]. An application-level fallback is the link most
likely to want the event, because it is the one link that has no idea
*where* the command came from until it reads
[`UiEvent::target`].
