The decision behind [`global_coordinates_are_published`], over plain positions
so it is reachable without an event loop (a `MonitorHandle` needs one, which is
why the rule this replaced was never tested).
