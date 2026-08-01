Moves history forward one entry and re-resolves it, or returns `None`
when already at the end (spec §6.13).

Like [`back`](Router::back), the entry is re-resolved against the current
registry, so a now-unregistered pattern yields a
[`NavigationResult::NotFound`].
