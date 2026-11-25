The largest motion step a single turn of the loop may advance by.

A panel that was asleep for a minute would otherwise wake and advance every
animation by a minute in one step, which reads as a jump rather than motion.

Per *turn*, not per frame, because that is where the loop reads the clock: see
[`TurnClock`](super::turn_clock::TurnClock), which is what makes the two the
same total.
