What one flex item answers when it is *asked* for a main-axis size.

The intrinsic half of flex item sizing, and the only part of it that
measures: css-flexbox-1 §4.5's content-based minimum and §9.2.3's `auto`
basis both need a size the item reports rather than one the style states, and
CSS Sizing 3 §4.1's `min-content`/`max-content`/`fit-content` keywords need
the same question asked with a different offer.

One primitive answers all of them — [`probe_main`], the item measured under a
stated main-axis offer — and the two functions above it are that offer chosen
twice: unbounded for a max-content answer, zero for a min-content one. The
narrow interface is the reason this is an entity: callers ask
[`intrinsic_main`] or [`content_max_main`] and never learn what a probe
offers, what percentage basis it states, or why.
