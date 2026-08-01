A coalesced layout change reported by a [`LayoutObserver`] (spec §4.4).

`previous` is `None` on the first observation and `Some` thereafter;
`current` is always the newly-committed box.
