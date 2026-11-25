How a started motion plays: its start delay, how many iterations it runs,
which direction iteration 1 takes, whether iterations alternate, and whether
the value fills the delay.

[`default`](Self::default) is every CSS initial value — no delay, exactly one
forward iteration, no alternation, no fill — which is what
[`UiRuntime::animate`](crate::UiRuntime::animate) starts a motion with.

The direction plus alternation pair models `animation-direction` exactly:
`alternate` is `Forward` + alternating and `alternate-reverse` is `Reverse` +
alternating, so **iteration 1** takes the named direction. That is the
spec's one-based parity — css-animations-1 §3.6: "For the purpose of
determining whether an iteration is even or odd, iterations start counting
from 1."
