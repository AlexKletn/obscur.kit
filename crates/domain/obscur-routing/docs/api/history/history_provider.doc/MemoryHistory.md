The reference, in-memory [`HistoryProvider`] (spec §6.13).

`MemoryHistory` keeps the visited entries in a [`Vec`] and a `cursor` index
pointing at the current entry. The cursor is only meaningful when the stack
is non-empty; when `entries` is empty it is a dormant `0` and every query
guards on emptiness. Fresh ids come from a plain monotonic `next_id` counter
— there is no wall-clock or randomness, so the provider is fully
deterministic.
