One winit monitor's readings, taken at a single instant (spec §6.6).

A [`MonitorHandle`](winit::monitor::MonitorHandle) cannot be constructed
without a live event loop and a real platform connection, so a projection
written directly against it is reachable only from a running winit
application. This snapshot is the seam that splits that problem in two: it
carries exactly the readings the projection consumes, in winit's own `dpi`
types, so [`DisplayConverter::project`] — the half with the decisions in it —
is an ordinary pure function, and [`MonitorSnapshot::read`] is left holding
nothing but five accessor calls.
