Moves history back one entry and re-resolves it, or returns `None` when
already at the start (spec §6.13).

The new current entry's route is re-resolved against the *current*
registry, so an entry whose pattern is no longer registered yields a
[`NavigationResult::NotFound`] rather than a stale match.
