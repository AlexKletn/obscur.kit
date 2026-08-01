Returns `true` if this policy continues from the element's current value
rather than snapping to a fresh start (spec §4.17 "interruption").

True for [`Blend`](Self::Blend) and [`Reverse`](Self::Reverse), which
both carry the present mid-animation state forward; false for
[`Restart`](Self::Restart) and [`Ignore`](Self::Ignore).
