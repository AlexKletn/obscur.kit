The size this event carries, if it carries one.

Both [`Configured`](Self::Configured) and [`Resized`](Self::Resized) do, and a
caller that treats them alike — which is most of them, since both mean "the
geometry you were drawing for is stale" — should not have to match twice.
