Returns whether the driver is still producing motion (spec §4.17).

The default is `true`: a driver keeps going unless it can settle or
finish. Drivers that reach a terminal state — such as a
[`TimeDriver`](super::TimeDriver) whose elapsed time has caught up to its
duration — override this to report when they have stopped advancing.
