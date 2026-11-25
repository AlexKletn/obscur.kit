A gradient whose color varies linearly along the axis from `start` to `end`
in user space (spec §5.1).

The gradient runs from `start` (offset `0.0`) to `end` (offset `1.0`); the
color at any point is found by projecting it onto that axis. Behavior for
projected positions outside `[0, 1]` is governed by the [`ExtendMode`].

Stops are stored in ascending `offset` order (callers supply them sorted;
see [`GradientStop`]).
