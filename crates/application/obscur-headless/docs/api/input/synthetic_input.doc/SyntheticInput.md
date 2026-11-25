A typed synthetic input event to inject into a headless run (spec §7.4).

Each variant is inert typed data describing one atomic input. The events are
injected into a headless run and dispatched — hit-tested, routed through
capture → target → bubble, and turned into a
[`DispatchOutcome`](crate::input::DispatchOutcome) — by
[`SyntheticDispatcher`](crate::input::SyntheticDispatcher), which replays them
through the runtime's real input path.
