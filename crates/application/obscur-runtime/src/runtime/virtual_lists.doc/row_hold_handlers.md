Long-press handlers for the rows of each virtual list, keyed by list id
(spec §4.9, §6.16). Same reason the click handlers exist: a realized row
cannot carry a per-row binding, so only the runtime can say which row a
held finger is on.
