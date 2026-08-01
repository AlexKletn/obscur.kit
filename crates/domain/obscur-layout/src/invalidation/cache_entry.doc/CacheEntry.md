One node's cached layout, stored by [`LayoutCache`](super::LayoutCache)
(spec §4.3).

It records the [`BoxConstraints`] the node was measured under together with
the resulting [`LayoutResult`], an optional arranged [`LayoutBox`], and the
[`VersionStamp`] at which it was produced. The stored constraints are the
cache key for the **early-exit on stable geometry**: a later measure request
hits only when its constraints equal the ones recorded here.

Because [`BoxConstraints`], [`LayoutResult`], and [`LayoutBox`] hold `f32`,
this type is `PartialEq` but not `Eq`.
