Builds a [`PointerEventKind::Up`] event at `position` for pointer `id`, with
`buttons` still held **after** the release (spec §4.7). See
[`pressed`](Self::pressed) for why the trusted flag needs this.
