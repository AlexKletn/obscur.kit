Clamps a resolved content-axis `extent` into the authored `min`/`max` for
that axis.

Each bound is resolved against `available` and mapped through `box_sizing`
(an authored min/max is a border-box length under `border-box`), matching how
the explicit size is resolved. An [`Auto`](Dimension::Auto) bound imposes no
limit. The max is applied before the min, so a min larger than the max wins
(per CSS). The result is never negative.

An **intrinsic** bound (`min-width: min-content`, …) has no arithmetic value,
so this — which only resolves — imposes no limit for one. A formatting context
that can measure its content passes the measured value through
[`clamp_content_measured`]; one that cannot degrades the bound to `auto`, the
same fallback [`Dimension::resolve`] gives every algorithm without intrinsic
support.
