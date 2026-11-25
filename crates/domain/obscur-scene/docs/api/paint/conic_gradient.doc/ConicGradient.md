A gradient whose color varies with the angle around `center` (spec §5.1).

The sweep begins at `start_angle` (in radians) and advances **clockwise**,
mapping a full turn of `2π` onto the `[0.0, 1.0]` offset range: offset `0.0`
lies along `start_angle` and offset `1.0` returns to it after one full
revolution. Behavior for offsets outside `[0, 1]` — which only arises when
stops themselves fall outside that range — is governed by the
[`ExtendMode`].

Stops are stored in ascending `offset` order (callers supply them sorted;
see [`GradientStop`]).
