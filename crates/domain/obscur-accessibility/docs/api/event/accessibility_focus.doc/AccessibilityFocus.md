An accessibility focus change (spec §7.1 focus).

Reports that accessibility focus moved from [`previous`](Self::previous) to
[`focused`](Self::focused). A `None` [`focused`](Self::focused) means focus
was cleared; a `None` [`previous`](Self::previous) means nothing was focused
before. Adapters reflect the new focus to the platform so assistive tech can
follow it.
