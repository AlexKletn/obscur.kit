Builds a [`PointerEventKind::Up`] event at `position` for pointer `id`, naming
the `button` that came up and the `held` set still down **after** the release
(spec §4.7). See [`pressed`](Self::pressed) for why the two are separate facts,
and for why the trusted flag needs this builder at all.
