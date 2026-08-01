A single input event from a gamepad (spec §6.16).

Each event pairs the [`gamepad`](Self::gamepad) that produced it with a
[`kind`](Self::kind) of change and a logical [`timestamp`](Self::timestamp)
supplied by the caller — this crate never reads a clock.
