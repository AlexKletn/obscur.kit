A motion value sampled in the **post-layout** phase (spec §4.17).

The counterpart to [`PreLayoutSample`](super::PreLayoutSample): a
`PostLayoutSample` carries a *presentation-only* animated value — a transform
or an opacity — that is applied **after** the layout pass and does **not**
change geometry (spec §4.17 "after-layout geometry"). Because it runs after
layout, the element's final laid-out box is available: an optional
[`post_layout_bounds`](Self::post_layout_bounds) captures that box for
post-layout transforms and shared-element work, and is `None` when no bounds
were captured.

The sample records which [`Timeline`](crate::timeline::Timeline) produced it,
the timeline [`progress`](Self::progress) at the moment of sampling, the
resolved [`MotionValue`](Self::value), and the optional captured bounds.
