Finds an installed face by its **named** family, or `None` when the host
does not have it.

# Why not `fc-match`

`fc-match` never fails: asked for a family that is not installed it returns
the system default with exit status 0. Built on it, this would report every
absent family as present under a substituted face — `font-family: Inter,
sans-serif` would resolve "Inter" to Noto Sans on a machine with no Inter,
and the author's fallback chain would never be walked.

Two things prevent that, and they are **redundant** — either alone suffices,
so neither should be read as the sole guard: `fc-list` returns nothing for
an absent family, and the name filter below rejects a face whose family is
not the one asked for. The filter is what makes the guarantee independent of
which tool is wired; `fc-list` is what leaves it nothing to reject.

The independent reason to prefer `fc-list` is **attribute choice**: it lists
every installed cut, so the nearest to `attributes` by
[`FontAttributes::distance`] is picked here. `fc-match` returns a single
face, which would make that fontconfig's decision rather than this
toolkit's.
